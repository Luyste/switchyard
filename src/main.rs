mod domain;
mod ps;
mod tmux;

use crate::{
    ps::{find_agent, list_processes},
    tmux::list_panes,
};

fn main() -> anyhow::Result<()> {
    let panes = list_panes()?;
    let processes = list_processes()?;

    for pane in &panes {
        if let Some((_, kind)) = find_agent(pane.pane_pid, &processes) {
            println!(
                "{} {} {}",
                pane.session_name,
                kind.label(),
                pane.pane_current_path.display()
            )
        };
    }

    Ok(())
}
