use anyhow::{Context, bail};
use std::{ffi::OsStr, path::Path, process::Command};

use crate::domain::AgentKind;

#[derive(Debug, PartialEq)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub command: String,
}

pub fn list_processes() -> anyhow::Result<Vec<Process>> {
    let output = Command::new("ps")
        .args(["-A", "-o", "pid=,ppid=,command="])
        .output()
        .context("failed to run ps")?;

    if !output.status.success() {
        bail!("ps failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let text = String::from_utf8(output.stdout).context("ps output is not UTF-8")?;

    Ok(parse_ps(&text))
}

pub fn parse_ps(output: &str) -> Vec<Process> {
    let mut processes: Vec<Process> = Vec::new();
    let lines = output.lines();

    for line in lines {
        let (pid, rest) = match line.trim_start().split_once(' ') {
            Some(pair) => pair,
            None => continue,
        };

        let (ppid, rest) = match rest.trim_start().split_once(' ') {
            Some(pair) => pair,
            None => continue,
        };

        let command = rest.trim_start();

        let pid = match pid.parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };

        let ppid = match ppid.parse::<u32>() {
            Ok(ppid) => ppid,
            Err(_) => continue,
        };

        let process = Process {
            pid,
            ppid,
            command: command.to_string(),
        };

        processes.push(process);
    }

    processes
}

// checks wheter or not the process runs an agent instance (pi/ claude)
pub fn runs(command: &str, name: &str) -> bool {
    let mut words = command.split_whitespace();

    let first = match words.next() {
        Some(first) => first,
        None => return false,
    };

    // the program itself: "pi" or "/usr/local/bin/pi"
    if file_name_is(first, name) {
        return true;
    }

    // an interpreter running the script: "node /opt/homebrew/bin/pi"
    if file_name_is(first, "node") {
        let second = match words.next() {
            Some(second) => second,
            None => return false,
        };

        return file_name_is(second, name);
    }

    false
}

// checks wether or not the file name is what we want
fn file_name_is(word: &str, want: &str) -> bool {
    // compare only the last path part, so "/opt/homebrew/bin/pi" counts as "pi"
    Path::new(word).file_name() == Some(OsStr::new(want))
}

fn agent_kind(command: &str) -> Option<AgentKind> {
    if runs(command, "pi") {
        return Some(AgentKind::Pi);
    }

    if runs(command, "claude") {
        return Some(AgentKind::Claude);
    }

    None
}

// finds the corresponding process and agent name from the process list based on pid
pub fn find_agent(pid: u32, processes: &[Process]) -> Option<(&Process, AgentKind)> {
    for child in processes.iter().filter(|p| p.ppid == pid) {
        if let Some(kind) = agent_kind(&child.command) {
            return Some((child, kind));
        }

        if let Some(found) = find_agent(child.pid, processes) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const PS_FIXTURES: &str = include_str!("../tests/fixtures/ps.txt");

    #[test]
    fn parses_ps_fixtures() {
        let ps = parse_ps(PS_FIXTURES);
        assert_eq!(ps.len(), 22);
        assert_eq!(
            ps[0],
            Process {
                pid: 1,
                ppid: 0,
                command: String::from("/sbin/launchd"),
            }
        );
    }

    #[test]
    fn skips_malformed_ps_lines() {
        const EMPTY_STRING: &str = "";
        let ps = parse_ps(EMPTY_STRING);

        assert_eq!(ps.len(), 0);

        // one good line; each bad line hits a different `continue`
        const MALFORMED: &str = "  42     1 /bin/zsh\n\
                                 abc 23145 /bin/zsh\n\
                                 12345\n\
                                 1 xyz foo";

        let ps = parse_ps(MALFORMED);

        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].pid, 42);
    }

    #[test]
    fn finds_agent() {
        let ps = parse_ps(PS_FIXTURES);
        assert_eq!(
            find_agent(17305, &ps).map(|p| (p.0.pid, p.1)),
            Some((17344, AgentKind::Pi))
        );
        assert_eq!(
            find_agent(1902, &ps).map(|p| (p.0.pid, p.1)),
            Some((1941, AgentKind::Pi))
        );
        assert_eq!(find_agent(71843, &ps).map(|p| (p.0.pid, p.1)), None);
        assert_eq!(
            find_agent(94991, &ps).map(|p| (p.0.pid, p.1)),
            Some((95013, AgentKind::Claude))
        );
    }

    #[test]
    fn runs_works() {
        let input = "node /opt/homebrew/bin/pi";
        assert!(runs(input, "pi"));

        let input = "zsh";
        assert!(!runs(input, "pi"));

        let input = "/bin/zsh -l -i -c pi ; exec zsh";
        assert!(!runs(input, "pi"));

        assert!(!runs("vim /tmp/pi", "pi"));

        assert!(runs("pi", "pi"));
        assert!(runs("/usr/local/bin/pi", "pi"));
    }
}
