use anyhow::{Context, bail};
use std::{path::PathBuf, process::Command};

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

pub const PANE_FORMAT: &str = "#{session_name}\t#{window_index}\t#{pane_index}\t#{pane_id}\t#{pane_pid}\t#{pane_current_command}\t#{pane_current_path}";

pub fn list_panes() -> anyhow::Result<Vec<Pane>> {
    let output = Command::new("tmux")
        .args(["list-panes", "-a", "-F", PANE_FORMAT])
        .output()
        .context("failed to run tmux")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_no_server(&stderr) {
            return Ok(Vec::new());
        }
        bail!("tmux failed: {stderr}");
    }

    let text = String::from_utf8(output.stdout).context("tmux output is not UTF-8")?;

    let panes = parse_panes(&text);

    Ok(panes)
}

fn is_no_server(stderr: &str) -> bool {
    stderr.starts_with("no server running") || stderr.starts_with("error connecting to")
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../tests/fixtures/tmux_list_panes.txt");

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
}
