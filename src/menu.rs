use crate::*;
use partition_navigation as pn;

#[derive(Debug, Clone)]
pub enum Provider {
    Remaining,
    Random,
    TopDownInversion,
    BottomUpInversion,
    TopDownJump,
    MaxInfoGain,
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
            Provider::MaxInfoGain => {
                Box::new(pn::providers::MaxInfoGain::new())
            }
            _ => panic!("Unimplemented!"),
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
            "TopDownJump" | "TDJ" => Ok(Self::TopDownJump),
            "MaxInfoGain" | "MIG" => Ok(Self::MaxInfoGain),
            _ => Err(format!("Unknown provider '{}'", s)),
        }
    }
}
