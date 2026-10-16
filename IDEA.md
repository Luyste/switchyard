# The idea behind sy

**The goal:** work in your editor (Zed first) while several coding agents (pi,
Claude Code) work next to you, each in its own copy of the repo, and manage
them all from one place: **the yard**, a terminal UI.

Each "because" below leads to a piece of sy.

## The foundation

**Because** agents must keep running when the editor closes, and Zed's AI
panel bills API tokens instead of your subscription,
**you need** agents to run in **tmux**, as the real `claude` / `pi` CLIs.
→ sy starts, attaches to, renames and kills tmux sessions.

**Because** several agents must work at once without touching each other's
files,
**you need** a **git worktree** per agent.
**Because** worktrunk (`wt`) already does that well (path template, config,
setup hooks like copying `.env`),
**you need** its library for the building blocks (git, config, hooks,
approvals), pinned to one version. Its commands (`wt switch`, hook running)
are private to the `wt` program, so sy writes small versions of those itself.

**Because** sy has to know which agent is where, without you telling it,
**you need discovery**: look at every tmux pane, walk down its processes and
find `pi` or `claude`.
→ `sy ls`, the base for everything else.

**Because** pi and Claude Code start, resume and fork differently,
**you need** one small description per agent: how to recognise it, start it,
continue it and fork it.

## The yard

**Because** you want one overview of every worktree and agent, and to start,
continue, fork and stop agents without typing commands,
**you need** a terminal UI: the yard. It runs in any terminal, so it works in
Zed, in nvim and on its own.

**Because** an editor task can't remember anything (each task is a new
process),
**you need** a **link**: "this editor folder talks to that agent", picked in
the yard and stored on disk.
→ `sy send` and `sy a` read the link instead of guessing.

**Because** you want the editor to go where your agent goes,
**you need** something that keeps watching. The yard does that while it's
open, and opens the editor in the new folder (`zed <path>`).

**Because** pi doesn't change its process's folder when it moves itself to a
worktree, so tmux can't see the move,
**you need** agents to report it themselves (`sy notify --cwd`, from a small
pi extension and Claude Code's hooks).

## From the editor

**Because** you want to show the agent code without copy-pasting,
**you need** to type text into the linked agent's pane, with the file and line
around it.
→ `sy send --file --line`, bound to a key in Zed.

## Later

**Because** you want to write a longer prompt about that code,
**you need** a prompt builder screen in the yard.

**Because** you want to know when an agent is working, waiting or done,
**you need** agents to report their state (`sy notify --state`).

## Today

`~/.config/zed/agent.sh` does attach, send and fork in bash, but it has to
guess which agent belongs to a folder. sy replaces it with discovery, links
and the yard, step by step.
