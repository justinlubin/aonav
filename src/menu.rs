use crate::*;
use partition_navigation as pn;

use serde::Serialize;
use strum::EnumString;

#[derive(Debug, Clone, Copy, EnumString, Serialize)]
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
    pub fn provider(
        &self,
    ) -> Box<dyn pbn::StepProvider<util::Timer, Step = pn::Step>> {
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
}
