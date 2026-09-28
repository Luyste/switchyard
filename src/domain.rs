#[derive(Debug, PartialEq)]
pub enum AgentState {
    Working,
    Waiting,
    Idle,
    Unknown,
}

pub struct Agent {
    pub name: String,
    pub kind: String,
    pub cwd: String,
    pub state: AgentState,
}

pub fn parse_state(s: &str) -> Option<AgentState> {
    match s {
        "working" => Some(AgentState::Working),
        "waiting" => Some(AgentState::Waiting),
        "idle" => Some(AgentState::Idle),
        _ => None,
    }
}

pub fn label(state: &AgentState) -> &'static str {
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
