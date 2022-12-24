use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_fundamental::AbsoluteSide;
use cetkaik_traits::CetkaikRepresentation;

pub struct HandExists_<T: CetkaikRepresentation> {
    pub if_tymok: GroundState_<T>,
    pub if_taxot: IfTaxot_<T>,
}

#[derive(Clone)]
pub enum TymokOrTaxot_<T: CetkaikRepresentation> {
    Tymok(GroundState_<T>),
    Taxot(IfTaxot_<T>),
}

pub trait CetkaikEngine<T: CetkaikRepresentation> {
    fn search(&mut self, s: &GroundState_<T>) -> Option<PureMove__<T::AbsoluteCoord>>;
    fn search_excited(
        &mut self,
        m: &InfAfterStep_<T::AbsoluteCoord>,
        s: &ExcitedState_<T>,
        ciurl: Option<usize>
    ) -> Option<AfterHalfAcceptance_<T::AbsoluteCoord>>;
    fn search_hand_resolved(&mut self, s: &HandExists_<T>) -> Option<TymokOrTaxot_<T>>;
}

fn score_gs<T: CetkaikRepresentation>(s: &GroundState_<T>) -> i32 {
    match s.whose_turn {
        AbsoluteSide::IASide => s.scores.ia() - s.scores.a(),
        AbsoluteSide::ASide => s.scores.a() - s.scores.ia(),
    }
}

pub fn score_hnr<T: CetkaikRepresentation>(s: &HandNotResolved_<T>) -> i32 {
    match s.whose_turn {
        AbsoluteSide::IASide => s.scores.ia() - s.scores.a(),
        AbsoluteSide::ASide => s.scores.a() - s.scores.ia(),
    }
}
