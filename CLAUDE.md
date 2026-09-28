# sy — context for Claude Code

`sy` is a Rust program that I'm building to **learn Rust**. It will slowly take
over the "systems" work of switchyard.nvim, my Neovim plugin for managing git
worktrees and coding agents (pi, Claude Code) running in tmux.

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

## Architecture (decided)

- **Domain model:** `Project`, `Worktree`, `Agent`, `AgentState`
  (`Working | Waiting | Idle | Unknown`), `Event`
  (`AgentStarted`, `AgentStopped`, `AgentMoved`, `AgentStateChanged`,
  `WorktreesChanged`).
- **Ports and adapters:** the core never calls programs directly; it talks to
  traits:
  - `Multiplexer` (tmux now; maybe Zellij later): panes, send, hand-over,
    new session, kill, rename.
  - `Worktrees`: list, create, remove. Goal implementation: `GitWorktrees`
    (`git worktree list --porcelain` / `add -b` / `remove`) plus sy's own
    **hooks**. `WorktrunkWorktrees` (parses `wt list --format=json`) stays as
    a backend during the migration. We do NOT reimplement worktrunk's other
    features (merge workflow, LLM commits, cache copying, PR checkout).
- **Hooks (the one worktrunk feature I need), worktrunk-compatible subset:**
  - Events: `pre-start` (blocking; `post-start` waits), `post-start`
    (background, logged), `pre-remove` (blocking; failure stops removal, runs
    in the worktree), `post-remove` (background).
  - Config: project hooks from the repo's `.config/wt.toml` (same file
    worktrunk reads, so both can coexist), user hooks from sy's config. Three
    forms like worktrunk: a command string, a table of named commands,
    `[[post-start]]` blocks (serde untagged enum).
  - Template variables: `branch`, `worktree_path`, `worktree_name`, `repo`,
    `repo_path`, `base`, `default_branch`, `primary_worktree_path`,
    `hook_type`, written as `{{ name }}`.
  - **Approvals are required:** project commands need approval on first run;
    store a hash per approved command in sy's state dir; changed command →
    ask again. Never run unapproved project hooks.
  - Background hooks: detached, output to log files, `sy logs`.
  - Caveat: pi-worktrunk (agents moving their own session between worktrees)
    needs `wt`; replacing it needs a small pi extension calling sy, and Claude
    Code's worktree hooks. Until then `wt` stays installed.
  - `AgentAdapter`: `matches(cmdline)`, `new_cmd`, `continue_cmd`,
    `fork_cmd(session)` (optional), `task_cmd(prompt)`.
  - Tests use fakes for these traits, plus fixtures (saved real output of
    tmux / wt / git) and `insta` snapshots.
- **Agent-agnostic transport (tmux only):**
  - Discovery: all tmux panes → walk down the process tree → match the command
    line against the adapters → the agent's cwd (`sysinfo`, or
    `lsof -a -p <pid> -d cwd -Fn` on macOS / `/proc/<pid>/cwd` on Linux).
  - Sending: `tmux load-buffer` + `paste-buffer -p` (bracketed paste) +
    `send-keys Enter`. Hand-over = without Enter.
  - Delivery check: `capture-pane` before and ~1–3 s after; unchanged → report
    "no response".
  - State: agents call `sy notify --pid <pid> --state working|waiting|idle
[--cwd <dir>]` from their hooks (Claude Code hooks, a small pi extension).
- **Daemon (milestone 2):** one **state actor** task owns the `World`; inputs
  (tmux control mode `tmux -C`, file watcher, `sy notify`, client requests)
  arrive on an `mpsc` channel; changes go out as `Event`s on a `broadcast`
  channel. No `Arc<Mutex<…>>` around the world.
- **Protocol:** JSON-RPC 2.0 over a Unix socket, newline-delimited, with a
  protocol version. Requests: `list`, `send`, `hand_over`, `new`, `kill`,
  `notify`; `subscribe` streams events. The CLI starts the daemon when needed.
- **Crates:** start as ONE crate with modules; split into a workspace only when
  it clearly helps.
- **Libraries:** `clap` (derive), `serde`/`serde_json`, `anyhow` (binary) +
  `thiserror` (library errors), `sysinfo`, `toml` + `directories`, later
  `tokio`, `notify`, `tracing`. Ask before adding anything else.

## The command line (sy is a tool of its own, not just a backend)

I want to manage agents from any terminal without typing tmux commands.

| Command                                                 | Does                                                                                                     |
| ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `sy new [agent] [--wt <branch>] [--task "…"] [-d]`      | start an agent in tmux (here or in a worktree, created with hooks if needed) and attach; `-d` = detached |
| `sy continue [agent]`                                   | resume the last session in this folder                                                                   |
| `sy fork [<ref>] <branch> [--task "…"] [--stop-source]` | fork an agent's conversation into another worktree; `--stop-source` = move (stop the source once idle)   |
| `sy ls` / `sy ls --json`                                | agents: name, agent, worktree, state, attached                                                           |
| `sy a [<ref>]`                                          | attach; inside tmux use `switch-client` instead of nesting                                               |
| `sy send <ref> "…"` / stdin                             | send a prompt; `--no-enter` = hand-over                                                                  |
| `sy kill <ref>`, `sy rename <ref> <name>`               | stop, rename                                                                                             |
| `sy wt ls` / `sy wt new <branch>` / `sy wt rm <branch>` | worktrees with hooks + approvals                                                                         |
| `sy notify --pid <pid> --state … [--cwd …]`             | for agent hooks                                                                                          |
| `sy` (no args)                                          | interactive picker: fzf when installed at first, later a `ratatui` TUI                                   |

- Refs: tmux name, unique prefix, number from `sy ls`, or `.` = agent in the
  current folder.
- Human-readable output by default, `--json` for scripts and switchyard.
- Shell completion via `clap_complete` (commands, agent names, branches).
- Config: `~/.config/sy/config.toml` (default agent, session naming,
  projects folders, terminal).
- Fork is the primitive; move = fork + stop the source when it's idle. For pi:
  `pi --fork <session file>`. For Claude Code, forking a session into a
  different folder still needs to be verified (sessions are stored per
  project folder).

## Milestones

0. Learning: Rust Book ch. 1–10, Rustlings, `cargo new sy`.
1. `sy ls` / `sy ls --json` (sync, no daemon): projects, worktrees, tmux
   panes, agents with cwd.
2. Agent sessions from the terminal: `sy new`, `sy continue`, `sy a`,
   `sy kill`, `sy rename`, `sy send` (+ `--no-enter`).
3. `sy wt ls/new/rm`: plain git, hooks, approvals, logs.
4. `sy fork` (+ `--stop-source`), callable by agents themselves.
5. `sy notify` + state hooks for Claude Code and pi.
6. Daemon: socket, tmux control mode, events.
7. `sy` without arguments: terminal UI with `ratatui`.
8. Release: `cargo-dist`, GitHub Actions, Homebrew tap.

The JSON that `sy ls --json` prints is consumed by switchyard.nvim (Lua), which
keeps a working Lua path until `sy` has proven itself (`backend = "lua" | "sy"`
per piece). Keep the JSON shape stable and versioned.

## Conventions

- `cargo fmt` and `cargo clippy` clean before a commit.
- Every parser has tests with real captured output in `tests/fixtures/`.
- Errors carry context (`anyhow::Context`): which command failed, with which
  arguments, and its stderr.
- No `unwrap()` outside tests, except where it's provably impossible to fail
  (and then a comment says why).
