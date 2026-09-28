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

impl AgentState {
    pub fn parse(s: &str) -> Option<AgentState> {
        match s {
            "working" => Some(AgentState::Working),
            "waiting" => Some(AgentState::Waiting),
            "idle" => Some(AgentState::Idle),
            _ => None,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            AgentState::Working => "working",
            AgentState::Waiting => "waiting",
            AgentState::Idle => "idle",
            AgentState::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; //import everything from the file above

    #[test]
    fn parses_known_states() {
        assert_eq!(AgentState::parse("working"), Some(AgentState::Working));
        assert_eq!(AgentState::parse("waiting"), Some(AgentState::Waiting));
        assert_eq!(AgentState::parse("idle"), Some(AgentState::Idle));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(AgentState::parse("banana"), None);
        assert_eq!(AgentState::parse("Idle"), None);
    }

    #[test]
    fn label_roundtrips() {
        for state in [AgentState::Working, AgentState::Waiting, AgentState::Idle] {
            assert_eq!(AgentState::parse(state.label()), Some(state));
        }

        let state = AgentState::Unknown;
        assert_eq!(AgentState::parse(state.label()), None);
    }
}
