use crate::*;

#[derive(Debug, Clone)]
pub enum Provider {
    CommittalAdd,
    CompleteRefine,
    ArbitrarySubsetCommit,
    Random,
}

impl Provider {
    pub fn provider(
        &self,
    ) -> Box<dyn pbn::StepProvider<Step = partition_navigation::Step>> {
        Box::new(pbn::CompoundProvider::new(vec![]))
        // TODO: add back in providers
        //  match self {
        //      Provider::CommittalAdd => {
        //          Box::new(navigation::providers::CommittalAddProvider::new())
        //      }
        //      Provider::CompleteRefine => {
        //          Box::new(navigation::providers::CompleteRefineProvider::new())
        //      }
        //      Provider::ArbitrarySubsetCommit => Box::new(
        //          navigation::providers::ArbitrarySubsetCommitProvider::new(),
        //      ),
        //      Provider::Random => {
        //          Box::new(navigation::providers::RandomProvider::new())
        //      }
        //  }
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
