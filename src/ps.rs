use std::{ffi::OsStr, path::Path};

#[derive(Debug, PartialEq)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub command: String,
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

pub fn is_pi(command: &str) -> bool {
    Path::new(command).file_name() == Some(OsStr::new("pi"))
}

pub fn find_agent(pid: u32, processes: &[Process]) -> Option<&Process> {
    for child in processes.iter().filter(|p| p.ppid == pid) {
        if is_pi(&child.command) {
            return Some(child);
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
        assert_eq!(find_agent(17305, &ps).map(|p| p.pid), Some(17344));
        assert_eq!(find_agent(1902, &ps).map(|p| p.pid), Some(1941));
        assert_eq!(find_agent(71834, &ps).map(|p| p.pid), None);
        assert_eq!(find_agent(94991, &ps).map(|p| p.pid), None);
    }

    #[test]
    fn is_pi_works() {
        let input = "node /opt/homebrew/bin/pi";
        assert!(is_pi(input));

        let input = "zsh";
        assert!(!is_pi(input));

        let input = "/bin/zsh -l -i -c pi ; exec zsh";
        assert!(!is_pi(input));
    }
}
