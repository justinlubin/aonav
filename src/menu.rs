//! # A menu of possible step providers
//!
//! This module hooks together all the components (ingredients) of this project
//! into a set of items on a menu of possible choices. To the extent possible,
//! these menu items all share the same interface and can be used in the same
//! way (e.g. as step providers for aonav).

use crate::*;
use partition_navigation as pn;

use serde::Serialize;
use strum::EnumString;

/// Menu of providers
#[derive(Debug, Clone, Copy, EnumString, Serialize)]
pub enum Provider {
    Remaining,
    Random,
    TopDownInversion,
    BottomUpInversion,
    MaxInfoGain,
    MaxInfoGainRelevant,
    MinLeafHeuristic,
    ForcedAssumptionsRemaining,
    AlphabeticalUnsound,
    AlphabeticalComplete,
    AlphabeticalRelevant,
    SufficiencySeeker,
    SufficiencySeekerRelevant,
}

impl Provider {
    /// Returns the provider for the selected menu item (optionally with an
    /// incremental starting expression)
    pub fn provider(
        &self,
        incremental_if_possible: Option<&pn::Exp>,
    ) -> Box<dyn pbn::StepProvider<util::Timer, Step = pn::Step>> {
        match self {
            Provider::Remaining => Box::new(pn::providers::Remaining::new(
                pn::oracle::OptInc::from_optional_start(
                    incremental_if_possible,
                ),
                false,
            )),
            Provider::Random => Box::new(pn::providers::Random::new(
                pn::oracle::OptInc::from_optional_start(
                    incremental_if_possible,
                ),
            )),
            Provider::TopDownInversion => {
                Box::new(pbn::CompoundProvider::new(vec![
                    Box::new(pn::providers::Commit::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                    )),
                    Box::new(pn::providers::TopDownInversion::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                    )),
                ]))
            }
            Provider::BottomUpInversion => {
                Box::new(pbn::FallbackProvider::new(vec![
                    Box::new(pn::providers::BottomUpInversion::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                    )),
                    Box::new(pn::providers::Leaf::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                    )),
                ]))
            }
            Provider::MaxInfoGain => Box::new(pn::providers::MaxInfoGain::new(
                pn::oracle::OptInc::from_optional_start(
                    incremental_if_possible,
                ),
                false,
            )),
            Provider::MaxInfoGainRelevant => {
                Box::new(pn::providers::MaxInfoGain::new(
                    pn::oracle::OptInc::from_optional_start(
                        incremental_if_possible,
                    ),
                    true,
                ))
            }
            Provider::MinLeafHeuristic => {
                Box::new(pn::providers::MinLeafHeuristic::new())
            }
            Provider::ForcedAssumptionsRemaining => {
                Box::new(pn::providers::ForcedAssumptions::new(Box::new(
                    pn::providers::Remaining::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                        false,
                    ),
                )))
            }
            Provider::AlphabeticalUnsound => {
                Box::new(pn::providers::Alphabetical::new(
                    pn::oracle::OptInc::from_optional_start(
                        incremental_if_possible,
                    ),
                    pn::providers::AlphabeticalMode::Unsound,
                ))
            }
            Provider::AlphabeticalComplete => {
                Box::new(pn::providers::Alphabetical::new(
                    pn::oracle::OptInc::from_optional_start(
                        incremental_if_possible,
                    ),
                    pn::providers::AlphabeticalMode::Complete,
                ))
            }
            Provider::AlphabeticalRelevant => {
                Box::new(pn::providers::Alphabetical::new(
                    pn::oracle::OptInc::from_optional_start(
                        incremental_if_possible,
                    ),
                    pn::providers::AlphabeticalMode::Relevant,
                ))
            }
            Provider::SufficiencySeeker => {
                Box::new(pn::providers::SufficiencySeeker::new(
                    Box::new(pn::providers::Remaining::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                        true,
                    )),
                    false,
                ))
            }
            Provider::SufficiencySeekerRelevant => {
                Box::new(pn::providers::SufficiencySeeker::new(
                    Box::new(pn::providers::Remaining::new(
                        pn::oracle::OptInc::from_optional_start(
                            incremental_if_possible,
                        ),
                        true,
                    )),
                    true,
                ))
            }
        }
    }
}
