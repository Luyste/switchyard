mod domain;
mod tmux;
use domain::{Agent, AgentState};

fn main() {
    let agent = Agent {
        name: String::from("pi-agent-1"),
        kind: String::from("pi"),
        cwd: String::from("/tmp/pi/"),
        state: AgentState::Working,
    };

    let agent2 = Agent {
        name: String::from("pi-agent-2"),
        kind: String::from("pi"),
        cwd: String::from("/tmp/pi2/"),
        state: AgentState::Unknown,
    };

    let parsed = AgentState::parse("banana");
    match parsed {
        Some(state) => println!("parsed: {}", state.label()),
        None => println!("None"),
    }

    println!(
        "{} {} {} {}",
        agent.name,
        agent.kind,
        agent.cwd,
        agent.state.label()
    );
    println!(
        "{} {} {} {}",
        agent2.name,
        agent2.kind,
        agent2.cwd,
        agent2.state.label()
    );

    println!("{:?}", agent.state);
}
