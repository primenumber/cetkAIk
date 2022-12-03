use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;
use cetkaik_core::absolute::Side;

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
    fn search_excited(&mut self, m: &InfAfterStep, s: &ExcitedState) -> Option<AfterHalfAcceptance>;
    fn search_hand_resolved(&mut self, s: &HandExists) -> Option<TymokOrTaxot>;
}

fn score_gs(s: &GroundState) -> i32 {
    match s.whose_turn {
        Side::IASide => s.scores.ia() - s.scores.a(),
        Side::ASide => s.scores.a() - s.scores.ia(),
    }
}

pub fn score_hnr(s: &HandNotResolved) -> i32 {
    match s.whose_turn {
        Side::IASide => s.scores.ia() - s.scores.a(),
        Side::ASide => s.scores.a() - s.scores.ia(),
    }
}
