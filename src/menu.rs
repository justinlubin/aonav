use crate::*;
use partition_navigation as pn;

#[derive(Debug, Clone)]
pub enum Provider {
    Remaining,
    Random,
    TopDownInversion,
    BottomUpInversion,
    MaxInfoGain,
    MinLeafHeuristic,
    ForcedAssumptions,
    AlphabeticalUnsound,
    AlphabeticalComplete,
    AlphabeticalRelevant,
}

impl Provider {
    pub fn provider(&self) -> Box<dyn pbn::StepProvider<Step = pn::Step>> {
        match self {
            Provider::Remaining => Box::new(pn::providers::Remaining::new()),
            Provider::Random => Box::new(pn::providers::Random::new()),
            Provider::TopDownInversion => {
                Box::new(pbn::CompoundProvider::new(vec![
                    Box::new(pn::providers::Commit::new()),
                    Box::new(pn::providers::TopDownInversion::new()),
                ]))
            }
            Provider::BottomUpInversion => {
                Box::new(pbn::FallbackProvider::new(vec![
                    Box::new(pn::providers::BottomUpInversion::new()),
                    Box::new(pn::providers::Leaf::new()),
                ]))
            }
            Provider::MaxInfoGain => {
                Box::new(pn::providers::MaxInfoGain::new())
            }
            Provider::MinLeafHeuristic => {
                Box::new(pn::providers::MinLeafHeuristic::new())
            }
            Provider::ForcedAssumptions => {
                Box::new(pn::providers::ForcedAssumptions::new())
            }
            Provider::AlphabeticalUnsound => {
                Box::new(pn::providers::Alphabetical::new(
                    pn::providers::AlphabeticalMode::Unsound,
                ))
            }
            Provider::AlphabeticalComplete => {
                Box::new(pn::providers::Alphabetical::new(
                    pn::providers::AlphabeticalMode::Complete,
                ))
            }
            Provider::AlphabeticalRelevant => {
                Box::new(pn::providers::Alphabetical::new(
                    pn::providers::AlphabeticalMode::Relevant,
                ))
            }
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Remaining" | "Re" => Ok(Self::Remaining),
            "Random" | "Ra" => Ok(Self::Random),
            "TopDownInversion" | "TDI" => Ok(Self::TopDownInversion),
            "BottomUpInversion" | "BUI" => Ok(Self::BottomUpInversion),
            // "TopDownJump" | "TDJ" => Ok(Self::TopDownJump),
            "MaxInfoGain" | "MIG" => Ok(Self::MaxInfoGain),
            "MinLeafHeuristic" | "MLH" => Ok(Self::MinLeafHeuristic),
            "ForcedAssumptions" | "FA" => Ok(Self::ForcedAssumptions),
            "AlphabeticalUnsound" | "AU" => Ok(Self::AlphabeticalUnsound),
            "AlphabeticalComplete" | "AC" => Ok(Self::AlphabeticalComplete),
            "AlphabeticalRelevant" | "AR" => Ok(Self::AlphabeticalRelevant),
            _ => Err(format!("Unknown provider '{}'", s)),
        }
    }
}
