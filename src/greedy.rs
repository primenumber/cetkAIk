use crate::cetkaik_engine::*;
use cetkaik_calculate_hand::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_fundamental::AbsoluteSide::{ASide, IASide};
use cetkaik_traits::CetkaikRepresentation;
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

    fn eval<T: CetkaikRepresentation>(&self, hnr_state: &HandNotResolved_<T>) -> f32 {
        let mut result = score_hnr(hnr_state) as f32;
        let (player_hop1zuo1, opponent_hop1zuo1) = match hnr_state.whose_turn {
            IASide => (
                T::hop1zuo1_of(IASide, &hnr_state.f),
                T::hop1zuo1_of(ASide, &hnr_state.f),
            ),
            ASide => (
                T::hop1zuo1_of(ASide, &hnr_state.f),
                T::hop1zuo1_of(IASide, &hnr_state.f),
            ),
        };
        result += 2.0
            * calculate_hands_and_score_from_pieces(&player_hop1zuo1)
                .unwrap()
                .score as f32;
        result += player_hop1zuo1.len() as f32;
        result
    }
}

impl<T: CetkaikRepresentation + Clone> CetkaikEngine<T> for GreedyPlayer {
    fn search(&mut self, s: &GroundState_<T>) -> Option<PureMove__<T::AbsoluteCoord>> {
        let (hop1zuo1_candidates, mut candidates) = s.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -50.0;
        candidates.shuffle(&mut self.rng);
        candidates.extend(hop1zuo1_candidates);
        for pure_move in candidates.iter() {
            let hnr_state = match pure_move {
                PureMove__::NormalMove(m) => {
                    match m {
                        NormalMove_::TamMoveNoStep { .. } => continue,
                        NormalMove_::TamMoveStepsDuringFormer { .. } => continue,
                        NormalMove_::TamMoveStepsDuringLatter { .. } => continue,
                        NormalMove_::NonTamMoveSrcStepDstFinite { src, step, dest } => {
                            if Some(T::absolute_tam2())
                                == T::absolute_get(T::as_board_absolute(&s.f), *step)
                                || src == dest
                            {
                                continue;
                            }
                        }
                        _ => (),
                    }
                    apply_normal_move::<T>(s, *m, self.config)
                        .unwrap()
                        .choose()
                        .0
                }
                PureMove__::InfAfterStep(m) => {
                    if Some(T::absolute_tam2())
                        == T::absolute_get(T::as_board_absolute(&s.f), m.src)
                    {
                        continue;
                    }
                    if Some(T::absolute_tam2())
                        == T::absolute_get(T::as_board_absolute(&s.f), m.step)
                    {
                        continue;
                    }
                    let ext_state = apply_inf_after_step(s, *m, self.config).unwrap().choose().0;
                    if let Some(aha_move) = self.search_excited(m, &ext_state) {
                        if aha_move.dest.is_none() {
                            continue;
                        }
                        apply_after_half_acceptance(&ext_state, aha_move, self.config)
                            .unwrap()
                            .choose()
                            .0
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

    fn search_excited(
        &mut self,
        m: &InfAfterStep_<T::AbsoluteCoord>,
        s: &ExcitedState_<T>,
    ) -> Option<AfterHalfAcceptance_<T::AbsoluteCoord>> {
        let candidates = s.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -50.0;
        for aha_move in candidates.iter() {
            if aha_move.dest == Some(m.src) {
                continue;
            }
            let hnr_state = apply_after_half_acceptance(s, *aha_move, self.config)
                .unwrap()
                .choose()
                .0;
            let score = self.eval(&hnr_state);
            if score > best_score {
                best_move = Some(aha_move);
                best_score = score;
            }
        }
        best_move.copied()
    }

    fn search_hand_resolved(&mut self, s: &HandExists_<T>) -> Option<TymokOrTaxot_<T>> {
        Some(TymokOrTaxot_::Taxot(s.if_taxot.clone()))
    }
}
