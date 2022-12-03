use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;

pub struct HandExists {
    pub if_tymok: GroundState,
    pub if_taxot: IfTaxot,
}

#[derive(Clone)]
pub enum TymokOrTaxot {
    Tymok(GroundState),
    Taxot(IfTaxot),
}

pub trait CetkaikEngine {
    fn search(&mut self, s: &GroundState) -> Option<PureMove>;
    fn search_excited(&mut self, s: &ExcitedState) -> Option<AfterHalfAcceptance>;
    fn search_hand_resolved(&mut self, s: &HandExists) -> Option<TymokOrTaxot>;
}

