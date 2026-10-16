# sy — context for Claude Code

`sy` is a Rust program that I'm building to **learn Rust**. It is switchyard
as a program of its own: it manages coding agents (pi, Claude Code) running in
tmux, each in its own git worktree. Its center is **the yard**, a terminal UI
that runs in any terminal: a Zed task, an nvim terminal, Ghostty. Editors only
call it; sy is editor-agnostic. Zed is the first editor it serves.

switchyard.nvim (`~/personal/projects/switchyard.nvim`) is the reference for
behaviour: its yard, linking and following are what sy rebuilds outside nvim.

See `IDEA.md` for the why behind each piece.

## Your role: tutor first, writer second

I have **zero prior Rust experience**. I come from full-stack TypeScript and
like strong types and interfaces. The goal is that I learn, not that code
appears.

- **I write the code.** Don't write or edit Rust files unless I explicitly ask
  you to ("write this", "implement it", "fix it for me").
- **Explain compiler errors** instead of fixing them: what the borrow checker
  or type checker is saying, why, and which options I have. Point me to the
  relevant chapter of the Rust Book when it helps.
- **Review what I write:** idiomatic Rust, error handling, naming, tests. Say
  what you'd change and why; let me make the change.
- **Relate new concepts to TypeScript** when it helps (enums with data ↔
  discriminated unions, traits ↔ interfaces, `Result` ↔ typed errors instead
  of exceptions, `Option` ↔ `T | undefined` that the compiler enforces).
- **One concept at a time.** Don't introduce async, lifetimes or advanced
  traits before the milestone needs them.
- **Clone freely early on.** Prefer clear code with `.clone()` over clever
  borrowing; we optimize later, if ever.
- When I do ask you to write code, keep it small, commented where it teaches
  something, and explain it afterwards.

## The yard (the center of sy)

`sy` without arguments opens the yard: two views, **worktrees** and
**agents**, single-key actions, `/` to filter (like switchyard.nvim's yard).

- **Actions:** worktrees: new, remove, open in the editor. Agents: start,
  continue, resume an earlier session (history), fork into another worktree,
  rename, stop, view (attach).
- **Link:** the agent you pick in the yard is linked to the editor folder you
  work in. The link is stored (`~/.local/state/sy/links.json`), so editor
  tasks like `sy send` know where to send without guessing.
- **Following:** while open, the yard looks at tmux every ~2 s (like
  switchyard.nvim) and sees agents start, stop and move. When the linked agent
  moves, the yard opens the editor there through the editor's own command
  line: `zed <path>`, or for nvim `nvim --server <socket> --remote-send`.
- **No daemon needed** for this: the yard is the long-running process while
  it's open. A daemon is only for later, if polling isn't enough.

## Zed: what the front-end can and can't do

The bash prototype `~/.config/zed/agent.sh` (with `tasks.json`,
`keymap.json`) is the **acceptance test** for the commands Zed calls: sy is
done for Zed when every task calls `sy` and the script is deleted.

| Key           | Today (`agent.sh` / Zed)     | With sy                                   |
| ------------- | ---------------------------- | ----------------------------------------- |
| `cmd-y`       | task picker                  | the yard (`sy`) as a task                 |
| `cmd-shift-l` | paste the selection          | `sy send . --no-enter --file … --line …`  |
| task picker   | attach / start an agent      | `sy a .` (the linked agent)               |
| task picker   | fork into a new worktree     | a yard action                             |

What Zed gives and doesn't:

- **Tasks are the only way in.** They pass `$ZED_WORKTREE_ROOT`,
  `$ZED_RELATIVE_FILE`, `$ZED_ROW`, `$ZED_SELECTED_TEXT` through `args` or
  `env` (`${VAR:}` = empty default, so the task isn't hidden without a
  selection). Tasks run in a terminal: the right dock, or the center. Zed has
  no floating terminals.
- **No input from tasks.** A task can't ask for anything; sy asks itself.
- **No UI from extensions** (only themes, languages, MCP, debuggers). The
  yard can only be a TUI in a Zed terminal.
- **No state between tasks.** Every task is a new process: that's why links
  are stored, and why following needs the yard.
- **No diagnostics** reach a task; only the cursor line (`$ZED_ROW`), not the
  selection's range.

## Architecture (decided)

- **Domain model:** `Project`, `Worktree`, `Agent`, `AgentState`
  (`Working | Waiting | Idle | Unknown`), `Link`, `Event`
  (`AgentStarted`, `AgentStopped`, `AgentMoved`, `AgentStateChanged`,
  `WorktreesChanged`).
- **Ports and adapters:** the core never calls programs directly; it talks to
  traits:
  - `Multiplexer` (tmux now; maybe Zellij later): panes, send, hand-over,
    new session, kill, rename, attach.
  - `Worktrees`: list, create, remove — **on top of the `worktrunk` library
    crate, pinned exactly** (`worktrunk = "=0.80.0"`; its docs say the API
    isn't stable, so we never update it casually). The library gives the
    building blocks; the *commands* are ours, because `wt switch`, `wt
    remove` and hook running live in the private `commands` module of the
    `wt` binary (`src/main.rs`), not in `src/lib.rs`.
    - From the library: `git` (`Repository`, `WorktreeInfo`, listing,
      `remove_worktree_with_cleanup`), `config` (the user and project config
      that `wt config` writes, `HooksConfig`, `Command`, template
      `expansion`, `Approvals`), `shell_exec`, `path`.
    - Ours (small versions of wt's commands, reading wt's source as the
      reference): **create** = render the `worktree-path` template, `git
      worktree add`, then run the hooks; **switch** = open the worktree in the
      editor; **remove** = pre-remove hooks, then the library's removal.
    - Hooks: the same `.config/wt.toml` and user config as `wt`, so `wt` and
      sy keep working side by side. **Never run an unapproved project hook**:
      check and record approvals through the library's `Approvals`, the same
      store `wt` uses. Background hooks (`post-start`) run detached with
      their output in a log file.
    - Keep a thin seam (`Worktrees` trait) around it, so a worktrunk update
      only touches one module.
  - `Editor`: open a folder (`zed <path>`, nvim through `--server`).
  - `AgentAdapter`: `matches(cmdline)`, `new_cmd`, `continue_cmd`,
    `fork_cmd(session, note)` (optional), `history(cwd)`, `task_cmd(prompt)`.
  - Tests use fakes for these traits, plus fixtures (saved real output of
    tmux / ps / wt) and `insta` snapshots.
- **Agent-agnostic transport (tmux only):**
  - Discovery: all tmux panes → walk down the process tree (`ps -A -o
    pid=,ppid=,command=`) → match the command line against the adapters →
    the pane's folder. Match the program's name, or an interpreter running a
    script of that name: pi runs as `node /opt/homebrew/bin/pi`. Compare
    folders resolved (`/tmp` is `/private/tmp` on macOS).
  - **Moves:** pi never changes its process's folder (its only `chdir` is a
    temporary one while spawning). After pi moves itself to a worktree
    (`wt switch`), tmux still reports the old folder. So a move must be
    **reported by the agent**: `sy notify --cwd <dir>` from a small pi
    extension and from Claude Code's hooks. tmux's folder is the fallback.
  - Starting: through a login shell, so the agent gets my PATH:
    `tmux new-session -d -s <name> -c <dir> $SHELL -l -i -c "<cmd>; exec $SHELL"`
    (a shell stays when the agent exits). Session names: `<agent>-<branch>`
    (`<branch>` = the folder name after its first `.`), at most 32
    characters, `.`, `:` and spaces become `_`, `-2`, `-3`, … when taken.
  - Sending: `tmux load-buffer` + `paste-buffer -p` (bracketed paste) +
    `send-keys Enter`. Hand-over = without Enter.
  - Delivery check: `capture-pane` before and ~1–3 s after; unchanged → report
    "no response".
  - State: agents call `sy notify --pid <pid> --state working|waiting|idle
    [--cwd <dir>]` from their hooks.
- **Daemon (later, optional):** one **state actor** owns the `World`; inputs
  (tmux control mode `tmux -C`, `sy notify`, client requests) arrive on an
  `mpsc` channel; changes go out as `Event`s on a `broadcast` channel. JSON-RPC
  2.0 over a Unix socket. Only when the polling yard isn't enough.
- **Crates:** start as ONE crate with modules; split into a workspace only when
  it clearly helps.
- **Libraries:** `clap` (derive), `serde`/`serde_json`, `anyhow` (binary) +
  `thiserror` (library errors), `ratatui` (with its `crossterm` backend),
  `worktrunk` (pinned `=0.80.0`), `directories`, later `insta`, `tokio`,
  `tracing`. Ask before adding
  anything else.

## The command line

The yard does most things; these commands are for editor tasks, scripts and
agents themselves.

| Command                                              | Does                                                                                               |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `sy`                                                 | the yard                                                                                           |
| `sy ls` / `sy ls --json`                             | agents: session, agent, folder, state, linked                                                      |
| `sy a [<ref>]`                                       | attach; `.` = the agent linked to this folder (the yard picks when there's none); inside tmux use `switch-client` |
| `sy send <ref> "…"` / stdin                          | send a prompt; `--no-enter` = hand-over; `--file <path> --line <n>` wraps it as context            |
| `sy new [agent] [--wt <branch>] [--task "…"]`        | start an agent (here or in a new worktree, created with hooks)                                     |
| `sy fork [<ref>] <branch> [--task "…"]`              | fork an agent's conversation into a new worktree                                                   |
| `sy notify --pid <pid> [--state …] [--cwd …]`        | for agent hooks: state and moves                                                                   |

- Refs: tmux name, unique prefix, number from `sy ls`, or `.` = the agent
  linked to the current folder.
- Human-readable output by default, `--json` for scripts. Keep the JSON shape
  stable and versioned.
- Config: `~/.config/sy/config.toml` (default agent, editor command, agents).
- **Context format** (`--file`/`--line`), the same as switchyard.nvim:

  ````
  <prompt>

  From <file>:<line>:
  ```
  <selection>
  ```
  ````

- **Prompt builder** (later, a yard screen): a multi-line prompt, the
  context below it, the target agent in the title. Keys: `enter` send,
  `shift-enter` newline, `ctrl-p` paste only, `tab` other agent, `ctrl-d`
  drop context, `esc` cancel.
- **Fork** is the primitive; move = fork + stop the source when it's idle. The
  fork starts with a note: "this session was forked from <old> and now runs
  in <new>; absolute paths in the history refer to the original worktree".
  - pi: the newest `.jsonl` in `~/.pi/agent/sessions/--<path>--/` (`<path>` =
    the resolved folder without its leading `/`, `/` and `:` → `-`), then
    `pi --fork <file> "<note>"`.
  - Claude Code: the session id is `sessionId` in
    `~/.claude/sessions/<pid>.json` (pid of the `claude` process), then
    `claude --resume <id> --fork-session "<note>"`.

## Milestones

0. Learning: Rust Book ch. 1–10, Rustlings, `cargo new sy`. ✓
1. **Discovery:** `parse_panes` ✓, `parse_ps` + the process tree,
   `find_agent`, `sy ls` / `--json`.
2. **Links + the Zed commands:** the link store, `sy a`, `sy send
   --file/--line --no-enter`. Point `tasks.json` at `sy`.
3. **The yard, first screen:** agents list (ratatui), `enter` = attach,
   polling every ~2 s.
4. **Worktrees on the worktrunk library:** first list (library only), then
   our own create and remove with the path template, hooks and approvals;
   the worktrees view; open in the editor; start an agent in a worktree.
5. **Agent actions:** continue, history, fork, rename, stop. Delete
   `agent.sh`.
6. **Moves and state:** `sy notify`, a small pi extension, Claude Code hooks;
   the yard follows the linked agent into its new worktree.
7. Prompt builder screen.
8. Release: `cargo-dist`, GitHub Actions, Homebrew tap.
9. Maybe: daemon, nvim front-end on top of the same yard.

## Conventions

- `cargo fmt` and `cargo clippy` clean before a commit.
- Every parser has tests with real captured output in `tests/fixtures/`.
- Errors carry context (`anyhow::Context`): which command failed, with which
  arguments, and its stderr.
- No `unwrap()` outside tests, except where it's provably impossible to fail
  (and then a comment says why).
