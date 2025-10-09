use crate::*;

#[derive(Debug, Clone)]
pub enum Provider {
    CommittalAdd,
    CompleteRefine,
    ArbitrarySubsetCommit,
    Random,
}

impl Provider {
    pub fn provider<'a, A: Clone + 'a, O: Clone + 'a>(
        &self,
    ) -> Box<dyn pbn::StepProvider<Step = navigation::Step<A, O>> + 'a> {
        match self {
            Provider::CommittalAdd => {
                Box::new(navigation::CommittalAddProvider::new())
            }
            Provider::CompleteRefine => {
                Box::new(navigation::CompleteRefineProvider::new())
            }
            Provider::ArbitrarySubsetCommit => {
                Box::new(navigation::ArbitrarySubsetCommitProvider::new())
            }
            Provider::Random => Box::new(navigation::RandomProvider::new()),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CommittalAdd" | "CA" => Ok(Self::CommittalAdd),
            "CompleteRefine" | "CR" => Ok(Self::CompleteRefine),
            "ArbitrarySubsetCommit" | "ASC" => Ok(Self::ArbitrarySubsetCommit),
            "Random" | "R" => Ok(Self::Random),
            _ => Err(format!("Unknown provider '{}'", s)),
        }
    }
}
