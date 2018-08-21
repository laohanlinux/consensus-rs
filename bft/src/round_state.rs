use std::fmt;
use std::time::Instant;

struct RoundState {
    height:  i64,
    round: i32,
    step: RoundStep,
    start_time: Instant,
}

#[derive(Clone, Copy)]
enum RoundStep {
    NewHeight,
    NewRound,
    Propose,
    Prevote,
    PreveoteWait,
    Precommit,
    PrecommitWait,
    Commit,
}

impl fmt::Display for RoundStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            RoundStep::NewHeight => {
                write!(f, "RoundStep::NewHeight").unwrap();
            },
            RoundStep::NewRound => {
                write!(f, "RoundStep::NewRound").unwrap();
            },
            RoundStep::Propose => {
                write!(f, "RoundStep::Propose").unwrap();
            },
            RoundStep::Prevote => {
                write!(f, "RoundStep::Prevote").unwrap();
            },
            RoundStep::PreveoteWait => {
                write!(f, "RoundStep::PreveoteWait").unwrap();
            },
            RoundStep::Precommit => {
                write!(f, "RoundStep::Precommit").unwrap();
            },
            RoundStep::PrecommitWait => {
                write!(f, "RoundStep::PreveoteWait").unwrap();
            }
            RoundStep::Commit => {
                write!(f, "RoundStep::Commit").unwrap();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    #[test]
    fn test_abc() {
        writeln!(io::stdout(), "{}", super::RoundStep::PrecommitWait).unwrap();
        writeln!(io::stdout(), "{}", super::RoundStep::Commit).unwrap();
    }
}