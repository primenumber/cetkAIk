use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;
use cetkaik_calculate_hand::*;
use cetkaik_core::absolute::Side;
use cetkaik_core::absolute::Piece;
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

    fn eval(&self, hnr_state: &HandNotResolved) -> f32 {
        let mut result = score_hnr(&hnr_state) as f32;
        let (player_hop1zuo1, opponent_hop1zuo1) = match hnr_state.whose_turn {
            Side::IASide => (&hnr_state.f.ia_side_hop1zuo1, &hnr_state.f.a_side_hop1zuo1),
            Side::ASide => (&hnr_state.f.a_side_hop1zuo1, &hnr_state.f.ia_side_hop1zuo1),
        };
        result += 2.0 * calculate_hands_and_score_from_pieces(&player_hop1zuo1).unwrap().score as f32;
        result += player_hop1zuo1.len() as f32;
        result
    }
}

impl CetkaikEngine for GreedyPlayer {
    fn search(&mut self, s: &GroundState) -> Option<PureMove> {
        let (hop1zuo1_candidates, mut candidates) = s.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -50.0;
        candidates.shuffle(&mut self.rng);
        candidates.extend(hop1zuo1_candidates);
        for pure_move in candidates.iter() {
            let hnr_state = match pure_move {
                PureMove::NormalMove(m) => {
                    match m {
                        NormalMove::TamMoveNoStep{..} => continue,
                        NormalMove::TamMoveStepsDuringFormer{..} => continue,
                        NormalMove::TamMoveStepsDuringLatter{..} => continue,
                        NormalMove::NonTamMoveSrcStepDstFinite{src, step, dest} => if let Some(Piece::Tam2) = s.f.board.get(&step) {
                            continue;
                        } else if src == dest {
                            continue;
                        }
                        _ => (),
                    }
                     apply_normal_move(&s, *m, self.config).unwrap().choose().0
                },
                PureMove::InfAfterStep(m) => {
                    if let Some(Piece::Tam2) = s.f.board.get(&m.src) {
                        continue;
                    }
                    if let Some(Piece::Tam2) = s.f.board.get(&m.step) {
                        continue;
                    }
                    let ext_state = apply_inf_after_step(&s, *m, self.config).unwrap().choose().0;
                    if let Some(aha_move) = self.search_excited(m, &ext_state) {
                        if aha_move.dest.is_none() {
                            continue;
                        }
                        apply_after_half_acceptance(&ext_state, aha_move, self.config).unwrap().choose().0
                    } else {
                        continue;
                    }
                }
            };
            let score = self.eval(&hnr_state);
            if score > best_score {
                best_move = Some(pure_move);
                best_score = score;
            }
        }
        best_move.cloned()
    }

    fn search_excited(&mut self, m: &InfAfterStep, s: &ExcitedState) -> Option<AfterHalfAcceptance> {
        let candidates = s.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -50.0;
        for aha_move in candidates.iter() {
            if aha_move.dest == Some(m.src) {
                continue;
            }
            let hnr_state = apply_after_half_acceptance(&s, *aha_move, self.config).unwrap().choose().0;
            let score = self.eval(&hnr_state);
            if score > best_score {
                best_move = Some(aha_move);
                best_score = score;
            }
        }
        best_move.copied()
    }

    fn search_hand_resolved(&mut self, s: &HandExists) -> Option<TymokOrTaxot> {
        Some(TymokOrTaxot::Taxot(s.if_taxot.clone()))
    }
}
