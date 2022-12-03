use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;
use crate::cetkaik_engine::*;
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct GreedyPlayer {
    config: Config,
    rng: SmallRng,
}

impl GreedyPlayer {
    pub fn new(config: Config) -> GreedyPlayer {
        GreedyPlayer {
            config,
            rng: SmallRng::from_entropy(),
        }
    }
}

impl CetkaikEngine for GreedyPlayer {
    fn search(&mut self, s: &GroundState) -> Option<PureMove> {
        let (hop1zuo1_candidates, mut candidates) = s.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -50;
        candidates.shuffle(&mut self.rng);
        candidates.extend(hop1zuo1_candidates);
        for pure_move in candidates.iter() {
            let hnr_state = match pure_move {
                PureMove::NormalMove(m) => {
                     apply_normal_move(&s, *m, self.config).unwrap().choose().0
                },
                PureMove::InfAfterStep(m) => {
                    let ext_state = apply_inf_after_step(&s, *m, self.config).unwrap().choose().0;
                    let aha_move = self.search_excited(&ext_state).unwrap();
                    apply_after_half_acceptance(&ext_state, aha_move, self.config).unwrap().choose().0
                }
            };
            let score = score_hnr(&hnr_state);
            if score > best_score {
                best_move = Some(pure_move);
                best_score = score;
            }
        }
        best_move.cloned()
    }

    fn search_excited(&mut self, s: &ExcitedState) -> Option<AfterHalfAcceptance> {
        let candidates = s.get_candidates(self.config);
        candidates.choose(&mut self.rng).copied()
    }

    fn search_hand_resolved(&mut self, s: &HandExists) -> Option<TymokOrTaxot> {
        Some(TymokOrTaxot::Taxot(s.if_taxot.clone()))
    }
}
