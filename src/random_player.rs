use crate::cetkaik_engine::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_traits::CetkaikRepresentation;
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct RandomPlayer {
    config: Config,
    rng: SmallRng,
}

impl RandomPlayer {
    pub fn new(config: Config) -> RandomPlayer {
        RandomPlayer {
            config,
            rng: SmallRng::from_entropy(),
        }
    }
}

impl<T: CetkaikRepresentation + Clone> CetkaikEngine<T> for RandomPlayer {
    fn search(&mut self, s: &GroundState_<T>) -> Option<PureMove__<T::AbsoluteCoord>> {
        let (hop1zuo1_candidates, candidates) = s.get_candidates(self.config);
        let pure_move_1 = hop1zuo1_candidates.choose(&mut self.rng);
        let pure_move_2 = candidates.choose(&mut self.rng);
        pure_move_1.or(pure_move_2).cloned()
    }

    fn search_excited(
        &mut self,
        _m: &InfAfterStep_<T::AbsoluteCoord>,
        s: &ExcitedState_<T>,
    ) -> Option<AfterHalfAcceptance_<T::AbsoluteCoord>> {
        let candidates = s.get_candidates(self.config);
        candidates.choose(&mut self.rng).copied()
    }

    fn search_hand_resolved(&mut self, s: &HandExists_<T>) -> Option<TymokOrTaxot_<T>> {
        [
            TymokOrTaxot_::Tymok(s.if_tymok.clone()),
            TymokOrTaxot_::Taxot(s.if_taxot.clone()),
        ]
        .choose(&mut self.rng)
        .cloned()
    }
}
