use crate::cetkaik_engine::*;
use cetkaik_calculate_hand::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::probabilistic::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_fundamental::AbsoluteSide;
use cetkaik_traits::{CetkaikRepresentation, IsBoard};

pub struct MinMaxPlayer {
    config: Config,
}

const SCORE_SCALE: i32 = 256;

impl MinMaxPlayer {
    pub fn new(config: Config) -> MinMaxPlayer {
        MinMaxPlayer { config }
    }

    fn eval<T: CetkaikRepresentation + Clone>(&self, state: &GroundState_<T>) -> i32 {
        let mut result = score_gs(&state);
        let (player_hop1zuo1, opponent_hop1zuo1) = match state.whose_turn {
            AbsoluteSide::IASide => (
                T::hop1zuo1_of(AbsoluteSide::IASide, &state.f),
                T::hop1zuo1_of(AbsoluteSide::ASide, &state.f),
            ),
            AbsoluteSide::ASide => (
                T::hop1zuo1_of(AbsoluteSide::ASide, &state.f),
                T::hop1zuo1_of(AbsoluteSide::IASide, &state.f),
            ),
        };
        result += 2 * calculate_hands_and_score_from_pieces(&player_hop1zuo1)
            .unwrap()
            .score * SCORE_SCALE;
        result -= 2 * calculate_hands_and_score_from_pieces(&opponent_hop1zuo1)
            .unwrap()
            .score * SCORE_SCALE;
        result += player_hop1zuo1.len() as i32 * SCORE_SCALE;
        result -= opponent_hop1zuo1.len() as i32 * SCORE_SCALE;
        result
    }

    fn eval_taxot<T: CetkaikRepresentation + Clone>(
        &self,
        player: AbsoluteSide,
        state: &IfTaxot_<T>,
    ) -> i32 {
        match state {
            IfTaxot_::NextSeason(p) => {
                let s = match p {
                    Probabilistic::WhoGoesFirst { ia_first, a_first } => {
                        if player == AbsoluteSide::IASide {
                            ia_first
                        } else {
                            a_first
                        }
                    }
                    _ => panic!("should not be given"),
                };
                score_gs(&s) * SCORE_SCALE
            }
            IfTaxot_::VictoriousSide(v) => if v.0 == Some(player) {
                40 * SCORE_SCALE
            } else {
                -40 * SCORE_SCALE
            },
        }
    }

    fn eval_prob_hand_not_resolved<T: CetkaikRepresentation + Clone>(
        &self,
        prob_state: &Probabilistic<HandNotResolved_<T>>,
        depth: usize,
        node_count: &mut usize,
    ) -> i32 {
        match prob_state {
            Probabilistic::Pure(k) => {
                let resolved = resolve(k, self.config);
                self.eval_hand_resolved_recursive(k.whose_turn, &resolved, depth, node_count)
            }
            Probabilistic::Water { failure, success } => {
                let resolved_failure = resolve(&failure, self.config);
                let resolved_success = resolve(&success, self.config);
                let sum = self.eval_hand_resolved_recursive(
                    failure.whose_turn,
                    &resolved_failure,
                    depth,
                    node_count,
                ) + self.eval_hand_resolved_recursive(
                    success.whose_turn,
                    &resolved_success,
                    depth,
                    node_count,
                );
                sum / 2
            }
            Probabilistic::Sticks { .. } => panic!("Sticks should not be given"),
            Probabilistic::WhoGoesFirst { .. } => panic!("WhoGoesFirst should not be given"),
        }
    }

    fn eval_prob_excited<T: CetkaikRepresentation + Clone>(
        &self,
        msg: &InfAfterStep_<T::AbsoluteCoord>,
        prob_state: &Probabilistic<ExcitedState_<T>>,
        depth: usize,
        node_count: &mut usize,
    ) -> i32 {
        match prob_state {
            Probabilistic::Pure(_) => panic!("Pure should not be given"),
            Probabilistic::Water { .. } => panic!("Water should not be given"),
            Probabilistic::Sticks {
                s0,
                s1,
                s2,
                s3,
                s4,
                s5,
            } => {
                //let sum = 1 * self.eval_excited_recursive(msg, &s0, Some(0), depth, node_count)
                //    + 5 * self.eval_excited_recursive(msg, &s1, Some(1), depth, node_count)
                //    + 10 * self.eval_excited_recursive(msg, &s2, Some(2), depth, node_count)
                //    + 10 * self.eval_excited_recursive(msg, &s3, Some(3), depth, node_count)
                //    + 5 * self.eval_excited_recursive(msg, &s4, Some(4), depth, node_count)
                //    + 1 * self.eval_excited_recursive(msg, &s5, Some(5), depth, node_count);
                let sum = 
                    1 * self.eval_excited_recursive(msg, &s1, Some(1), depth, node_count)
                    + 2 * self.eval_excited_recursive(msg, &s2, Some(2), depth, node_count)
                    + 2 * self.eval_excited_recursive(msg, &s3, Some(3), depth, node_count)
                    + 1 * self.eval_excited_recursive(msg, &s4, Some(4), depth, node_count);
                sum / 6
            }
            Probabilistic::WhoGoesFirst { .. } => {
                panic!("WhoGoesFirst should not be given")
            }
        }
    }

    fn eval_ground_recursive<T: CetkaikRepresentation + Clone>(
        &self,
        state: &GroundState_<T>,
        depth: usize,
        node_count: &mut usize,
    ) -> i32 {
        *node_count += 1;
        if depth == 0 {
            return self.eval(state);
        }
        let (hop1zuo1_candidates, mut candidates) = state.get_candidates(self.config);
        candidates.extend(hop1zuo1_candidates);
        candidates
            .iter()
            .filter_map(|pure_move| match pure_move {
                PureMove__::NormalMove(m) => {
                    match m {
                        NormalMove_::TamMoveNoStep { .. } => return None,
                        NormalMove_::TamMoveStepsDuringFormer { .. } => return None,
                        NormalMove_::TamMoveStepsDuringLatter { .. } => return None,
                        NormalMove_::NonTamMoveFromHopZuo { .. } => return None,
                        NormalMove_::NonTamMoveSrcStepDstFinite { src, step, dest } => {
                            if Some(T::absolute_tam2())
                                == T::as_board_absolute(&state.f).peek(*step)
                                || src == dest
                            {
                                return None;
                            }
                        }
                        _ => (),
                    }
                    Some(self.eval_prob_hand_not_resolved(
                        &apply_normal_move::<T>(state, *m, self.config).unwrap(),
                        depth,
                        node_count,
                    ))
                }
                PureMove__::InfAfterStep(m) => {
                    if Some(T::absolute_tam2()) == T::as_board_absolute(&state.f).peek(m.src) {
                        return None;
                    }
                    if Some(T::absolute_tam2()) == T::as_board_absolute(&state.f).peek(m.step) {
                        return None;
                    }
                    if m.src == m.planned_direction {
                        return None;
                    }
                    Some(self.eval_prob_excited(
                        m,
                        &apply_inf_after_step(state, *m, self.config).unwrap(),
                        depth,
                        node_count,
                    ))
                }
            })
            .max()
            .unwrap()
    }

    fn eval_excited_recursive<T: CetkaikRepresentation + Clone>(
        &self,
        _msg: &InfAfterStep_<T::AbsoluteCoord>,
        state: &ExcitedState_<T>,
        _ciurl: Option<usize>,
        depth: usize,
        node_count: &mut usize,
    ) -> i32 {
        *node_count += 1;
        let candidates = state.get_candidates(self.config);
        candidates
            .iter()
            .map(|aha_move| {
                self.eval_prob_hand_not_resolved(
                    &apply_after_half_acceptance(state, *aha_move, self.config).unwrap(), depth, node_count
                )
            })
            .max()
            .unwrap()
    }

    fn eval_hand_resolved_recursive<T: CetkaikRepresentation + Clone>(
        &self,
        whose_turn: AbsoluteSide,
        state: &HandResolved_<T>,
        depth: usize,
        node_count: &mut usize,
    ) -> i32 {
        *node_count += 1;
        match state {
            HandResolved_::NeitherTymokNorTaxot(s) => {
                -self.eval_ground_recursive(&s, depth - 1, node_count)
            }
            HandResolved_::HandExists { if_tymok, if_taxot } => std::cmp::max(
                -self.eval_ground_recursive(&if_tymok, depth - 1, node_count),
                self.eval_taxot(whose_turn, if_taxot),
            ),
            HandResolved_::GameEndsWithoutTymokTaxot(v) => if v.0 == Some(whose_turn) {
                40 * SCORE_SCALE
            } else {
                -40 * SCORE_SCALE
            },
        }
    }
}

impl<T: CetkaikRepresentation + Clone> CetkaikEngine<T> for MinMaxPlayer {
    fn search(&mut self, state: &GroundState_<T>) -> Option<PureMove__<T::AbsoluteCoord>> {
        let (hop1zuo1_candidates, mut candidates) = state.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -12800.0;
        candidates.extend(hop1zuo1_candidates);
        let depth = 2;
        let mut node_count = 0;
        for pure_move in candidates.iter() {
            let score = match pure_move {
                PureMove__::NormalMove(m) => {
                    match m {
                        NormalMove_::TamMoveNoStep { .. } => continue,
                        NormalMove_::TamMoveStepsDuringFormer { .. } => continue,
                        NormalMove_::TamMoveStepsDuringLatter { .. } => continue,
                        NormalMove_::NonTamMoveFromHopZuo { .. } => continue,
                        NormalMove_::NonTamMoveSrcStepDstFinite { src, step, dest } => {
                            if Some(T::absolute_tam2())
                                == T::as_board_absolute(&state.f).peek(*step)
                                || src == dest
                            {
                                continue;
                            }
                        }
                        _ => (),
                    }
                    self.eval_prob_hand_not_resolved(
                        &apply_normal_move::<T>(state, *m, self.config).unwrap(),
                        depth,
                        &mut node_count,
                    )
                }
                PureMove__::InfAfterStep(m) => {
                    if Some(T::absolute_tam2()) == T::as_board_absolute(&state.f).peek(m.src) {
                        continue;
                    }
                    if Some(T::absolute_tam2()) == T::as_board_absolute(&state.f).peek(m.step) {
                        continue;
                    }
                    if m.src == m.planned_direction {
                        continue;
                    }
                    self.eval_prob_excited(
                        m,
                        &apply_inf_after_step(state, *m, self.config).unwrap(),
                        depth,
                        &mut node_count,
                    )
                }
            } as f32;
            if score > best_score {
                best_move = Some(pure_move);
                best_score = score;
            }
        }
        eprintln!("Nodes: {}", node_count);
        best_move.cloned()
    }

    fn search_excited(
        &mut self,
        _msg: &InfAfterStep_<T::AbsoluteCoord>,
        state: &ExcitedState_<T>,
        _ciurl: Option<usize>,
    ) -> Option<AfterHalfAcceptance_<T::AbsoluteCoord>> {
        let candidates = state.get_candidates(self.config);
        let mut best_move = None;
        let mut best_score = -12800.0;
        let depth = 2;
        let mut node_count = 0;
        for aha_move in candidates.iter() {
            let score = self.eval_prob_hand_not_resolved(
                &apply_after_half_acceptance(state, *aha_move, self.config).unwrap(),
                depth, &mut node_count,
            ) as f32;
            if score > best_score {
                best_move = Some(aha_move);
                best_score = score;
            }
        }
        eprintln!("Nodes: {}", node_count);
        best_move.copied()
    }

    fn search_hand_resolved(&mut self, s: &HandExists_<T>) -> Option<TymokOrTaxot_<T>> {
        Some(TymokOrTaxot_::Taxot(s.if_taxot.clone()))
    }
}
