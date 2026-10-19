mod domain;
mod ps;
mod tmux;

use crate::{
    domain::{Agent, AgentState},
    ps::{find_agent, list_processes},
    tmux::list_panes,
};

fn main() -> anyhow::Result<()> {
    let agents = discover()?;

    for agent in agents {
        println!(
            "{} {} {}",
            agent.name,
            agent.kind.label(),
            agent.cwd.display()
        )
    }

    Ok(())
}

pub fn discover() -> anyhow::Result<Vec<Agent>> {
    let panes = list_panes()?;
    let processes = list_processes()?;
    let mut agents = Vec::<Agent>::new();

    for pane in panes {
        if let Some((_, kind)) = find_agent(pane.pane_pid, &processes) {
            let agent = Agent {
                name: pane.session_name,
                kind,
                cwd: pane.pane_current_path,
                state: AgentState::Unknown,
            };

            agents.push(agent);
        };
    }

    Ok(agents)
}
