mod domain;
use domain::{Agent, AgentState, label, parse_state};

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

    let parsed = parse_state("banana");
    match parsed {
        Some(state) => println!("parsed: {}", label(&state)),
        None => println!("None"),
    }

    println!(
        "{} {} {} {}",
        agent.name,
        agent.kind,
        agent.cwd,
        label(&agent.state)
    );
    println!(
        "{} {} {} {}",
        agent2.name,
        agent2.kind,
        agent2.cwd,
        label(&agent2.state)
    );

    println!("{:?}", agent.state);
}
