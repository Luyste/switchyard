use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Pane {
    pub session_name: String,
    pub window_index: u32,
    pub pane_index: u32,
    pub pane_id: String,
    pub pane_pid: u32,
    pub pane_current_command: String,
    pub pane_current_path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub command: String,
}

pub fn parse_panes(output: &str) -> Vec<Pane> {
    let mut panes: Vec<Pane> = Vec::new();
    let lines = output.lines();

    for line in lines {
        let fields = line.split('\t').collect::<Vec<&str>>();

        if fields.len() != 7 {
            continue;
        }

        let pane_pid = match fields[4].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };

        let window_index = match fields[1].parse::<u32>() {
            Ok(index) => index,
            Err(_) => continue,
        };

        let pane_index = match fields[2].parse::<u32>() {
            Ok(index) => index,
            Err(_) => continue,
        };

        let pane: Pane = Pane {
            session_name: String::from(fields[0]),
            window_index,
            pane_index,
            pane_id: String::from(fields[3]),
            pane_pid,
            pane_current_command: String::from(fields[5]),
            pane_current_path: PathBuf::from(fields[6]),
        };

        panes.push(pane);
    }

    panes
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../tests/fixtures/tmux_list_panes.txt");
    const PS_FIXTURES: &str = include_str!("../tests/fixtures/ps.txt");

    #[test]
    fn parses_panes_fixtures() {
        let panes = parse_panes(FIXTURES);
        assert_eq!(panes.len(), 6);
        assert_eq!(
            panes[0],
            Pane {
                session_name: String::from("claude-switchyard"),
                window_index: 0,
                pane_index: 0,
                pane_id: String::from("%9"),
                pane_pid: 94991,
                pane_current_command: String::from("2.1.301"),
                pane_current_path: PathBuf::from(
                    "/Users/jopluysterburg/personal/projects/switchyard"
                )
            }
        );
    }

    #[test]
    fn skips_malformed_panes_lines() {
        const FIXTURE: &str = "testpoopie\tblablabanana\t";
        let mut panes = parse_panes(FIXTURE);
        assert_eq!(panes.len(), 0);

        let input = "good\t0\t0\t%1\t123\tzsh\t/tmp\n\
               too\tshort\n\
               badpid\t0\t0\t%2\tabc\tzsh\t/tmp";

        panes = parse_panes(input);
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].session_name, String::from("good"));
    }

    #[test]
    fn parses_pid_fixtures() {
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
}
