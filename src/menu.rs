use crate::*;
use partition_navigation as pn;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Remaining,
    Random,
    TopDownInversion,
    BottomUpInversion,
    MaxInfoGain,
    MaxInfoGainRelevant,
    MinLeafHeuristic,
    ForcedAssumptions,
    AlphabeticalUnsound,
    AlphabeticalComplete,
    AlphabeticalRelevant,
}

impl Provider {
    pub fn all() -> &'static [Provider] {
        &[
            Provider::Remaining,
            Provider::Random,
            Provider::TopDownInversion,
            Provider::BottomUpInversion,
            Provider::MaxInfoGain,
            Provider::MaxInfoGainRelevant,
            Provider::MinLeafHeuristic,
            Provider::ForcedAssumptions,
            Provider::AlphabeticalUnsound,
            Provider::AlphabeticalComplete,
            Provider::AlphabeticalRelevant,
        ]
    }

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
                Box::new(pn::providers::MaxInfoGain::new(false))
            }
            Provider::MaxInfoGainRelevant => {
                Box::new(pn::providers::MaxInfoGain::new(true))
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

    pub fn shorthand(&self) -> &str {
        match self {
            Provider::Remaining => "Re",
            Provider::Random => "Ra",
            Provider::TopDownInversion => "TDI",
            Provider::BottomUpInversion => "BUI",
            Provider::MaxInfoGain => "MIG",
            Provider::MaxInfoGainRelevant => "MIGR",
            Provider::MinLeafHeuristic => "MLH",
            Provider::ForcedAssumptions => "FA",
            Provider::AlphabeticalUnsound => "AU",
            Provider::AlphabeticalComplete => "AC",
            Provider::AlphabeticalRelevant => "AR",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Remaining => write!(f, "Remaining"),
            Provider::Random => write!(f, "Random"),
            Provider::TopDownInversion => write!(f, "TopDownInversion"),
            Provider::BottomUpInversion => write!(f, "BottomUpInversion"),
            Provider::MaxInfoGain => write!(f, "MaxInfoGain"),
            Provider::MaxInfoGainRelevant => write!(f, "MaxInfoGainRelevant"),
            Provider::MinLeafHeuristic => write!(f, "MinLeafHeuristic"),
            Provider::ForcedAssumptions => write!(f, "ForcedAssumptions"),
            Provider::AlphabeticalUnsound => write!(f, "AlphabeticalUnsound"),
            Provider::AlphabeticalComplete => write!(f, "AlphabeticalComplete"),
            Provider::AlphabeticalRelevant => write!(f, "AlphabeticalRelevant"),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for provider in Provider::all() {
            if s == provider.shorthand() || s == provider.to_string() {
                return Ok(*provider);
            }
        }
        Err(format!("Unknown provider '{}'", s))
    }
}
