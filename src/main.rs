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

#[derive(Debug, PartialEq)]
enum AgentState {
    Working,
    Waiting,
    Idle,
    Unknown,
}

struct Agent {
    name: String,
    kind: String,
    cwd: String,
    state: AgentState,
}

fn parse_state(s: &str) -> Option<AgentState> {
    match s {
        "working" => Some(AgentState::Working),
        "waiting" => Some(AgentState::Waiting),
        "idle" => Some(AgentState::Idle),
        _ => None,
    }
}

fn label(state: &AgentState) -> &'static str {
    match state {
        AgentState::Working => "working",
        AgentState::Waiting => "waiting",
        AgentState::Idle => "idle",
        AgentState::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*; //import everything from the file above

    #[test]
    fn parses_known_states() {
        assert_eq!(parse_state("working"), Some(AgentState::Working));
        assert_eq!(parse_state("waiting"), Some(AgentState::Waiting));
        assert_eq!(parse_state("idle"), Some(AgentState::Idle));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_state("banana"), None);
        assert_eq!(parse_state("Idle"), None);
    }

    #[test]
    fn label_roundtrips() {
        for state in [AgentState::Working, AgentState::Waiting, AgentState::Idle] {
            assert_eq!(parse_state(label(&state)), Some(state));
        }

        let state = AgentState::Unknown;
        assert_eq!(parse_state(label(&state)), None);
    }
}
