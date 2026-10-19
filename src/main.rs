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
        if find_agent(pane.pane_pid, &processes).is_some() {
            println!(
                "{} pi {}",
                pane.session_name,
                pane.pane_current_path.display()
            )
        };
    }

    Ok(())
}
