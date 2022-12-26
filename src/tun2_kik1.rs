use cetkaik_calculate_hand::calculate_hands_and_score_from_pieces;
use cetkaik_full_state_transition::message::AfterHalfAcceptance_;
use cetkaik_full_state_transition::state::{ExcitedState_, HandNotResolved_, HandResolved_};
use cetkaik_full_state_transition::Config;
use cetkaik_full_state_transition::{apply_after_half_acceptance, apply_inf_after_step, resolve};
use cetkaik_full_state_transition::{
    apply_normal_move,
    message::{InfAfterStep_, NormalMove_, PureMove__},
    probabilistic::Probabilistic,
    state::GroundState_,
};
use cetkaik_fundamental::{ColorAndProf, Profession, PureMove_};
use cetkaik_traits::{CetkaikRepresentation, IsBoard, IsPieceWithSide};
use cetkaik_yhuap_move_candidates::{
    is_tam_hue_relative, not_from_hop1zuo1_candidates_vec, AllowKut2Tam2,
};

use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::cetkaik_engine::{score_hnr, CetkaikEngine, HandExists_, TymokOrTaxot_};

pub struct Tun2Kik1 {
    config: Config,
    rng: SmallRng,
    show_tactics_message: bool,
}

impl Tun2Kik1 {
    pub fn new(config: Config, show_tactics_message: bool) -> Tun2Kik1 {
        Tun2Kik1 {
            config,
            rng: SmallRng::from_entropy(),
            show_tactics_message,
        }
    }

    fn eval<T: CetkaikRepresentation>(&self, hnr_state: &HandNotResolved_<T>) -> f32 {
        let mut result = score_hnr(hnr_state) as f32;
        let player_hop1zuo1 = T::hop1zuo1_of(hnr_state.whose_turn, &hnr_state.f);
        result += 2.0
            * calculate_hands_and_score_from_pieces(&player_hop1zuo1)
                .unwrap()
                .score as f32;
        result += player_hop1zuo1.len() as f32;
        result
    }
}

impl<T: CetkaikRepresentation + Clone + std::fmt::Debug> CetkaikEngine<T> for Tun2Kik1 {
    fn search(&mut self, s: &GroundState_<T>) -> Option<PureMove__<T::AbsoluteCoord>> {
        let res = generate_move(&mut self.rng, self.config, s, s.tam_has_moved_previously);
        if self.show_tactics_message {
            println!("{}", res.tactics);
        }
        Some(res.bot_move.into())
    }

    fn search_excited(
        &mut self,
        m: &InfAfterStep_<T::AbsoluteCoord>,
        s: &ExcitedState_<T>,
        ciurl: Option<usize>,
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

fn is_victorious_hand<T: CetkaikRepresentation>(
    cand: PureMove_<T::AbsoluteCoord>,
    game_state: &GroundState_<T>,
) -> bool {
    match cand {
        PureMove_::NonTamMoveFromHopZuo { .. }
        | PureMove_::TamMoveNoStep { .. }
        | PureMove_::TamMoveStepsDuringFormer { .. }
        | PureMove_::TamMoveStepsDuringLatter { .. } => return false,

        PureMove_::InfAfterStep {
            src,
            step,
            planned_direction,
        } => {
            if planned_direction == src {
                // self-occlusion
                return false;
            }

            let dest_occupied_by = T::as_board_absolute(&game_state.f).peek(planned_direction);

            let Some(piece) = dest_occupied_by else {
                // cannot win if a piece was not obtained
                return false;
            };

            return piece.match_on_piece_and_apply(
                &|| panic!("tam cannot be captured, why is it in the destination?"),
                &|color, prof, side| {
                    let mut hop1zuo1 = T::hop1zuo1_of(game_state.whose_turn, &game_state.f);
                    let old_calc = calculate_hands_and_score_from_pieces(&hop1zuo1).unwrap();

                    hop1zuo1.push(ColorAndProf { color, prof });
                    let new_calc = calculate_hands_and_score_from_pieces(&hop1zuo1).unwrap();

                    return new_calc.score != old_calc.score;
                },
            );
        }

        PureMove_::NonTamMoveSrcDst { src, dest, .. }
        | PureMove_::NonTamMoveSrcStepDstFinite { src, dest, .. } => {
            if dest == src {
                // self-occlusion
                return false;
            }
            let dest_occupied_by = T::as_board_absolute(&game_state.f).peek(dest);

            let Some(piece) = dest_occupied_by else {
                // cannot win if a piece was not obtained
                return false;
            };

            return piece.match_on_piece_and_apply(
                &|| panic!("tam cannot be captured, why is it in the destination?"),
                &|color, prof, side| {
                    let mut hop1zuo1 = T::hop1zuo1_of(game_state.whose_turn, &game_state.f);
                    let old_calc = calculate_hands_and_score_from_pieces(&hop1zuo1).unwrap();

                    hop1zuo1.push(ColorAndProf { color, prof });
                    let new_calc = calculate_hands_and_score_from_pieces(&hop1zuo1).unwrap();

                    return new_calc.score != old_calc.score;
                },
            );
        }
    };
}

fn likely<T: CetkaikRepresentation>(
    cand: &PureMove__<T::AbsoluteCoord>,
    ciurl_threshold: i32,
) -> bool {
    match cand {
        PureMove__::InfAfterStep(InfAfterStep_ {
            src: _,
            step,
            planned_direction,
        }) => T::absolute_distance(*planned_direction, *step) <= ciurl_threshold,
        PureMove__::NormalMove(nm) => match nm {
            NormalMove_::NonTamMoveFromHopZuo { .. }
            | NormalMove_::TamMoveNoStep { .. }
            | NormalMove_::TamMoveStepsDuringFormer { .. }
            | NormalMove_::TamMoveStepsDuringLatter { .. } => true,
            NormalMove_::NonTamMoveSrcDst { dest, .. }
            | NormalMove_::NonTamMoveSrcStepDstFinite { dest, .. } => !T::is_water_absolute(*dest),
        },
    }
}

/// 「入水判定が要らず、3以下の踏越え判定しか要らない」を「やりづらくはない(likely to succeed)」と定義する。
pub fn is_likely_to_succeed<T: CetkaikRepresentation>(cand: &PureMove__<T::AbsoluteCoord>) -> bool {
    likely::<T>(cand, 3)
}

/// 「入水判定が要らず、2以下の踏越え判定しか要らない」を「やりやすい(very likely to succeed)」と定義する。
pub fn is_very_likely_to_succeed<T: CetkaikRepresentation>(
    cand: &PureMove__<T::AbsoluteCoord>,
) -> bool {
    likely::<T>(cand, 2)
}

fn if_capture_get_coord<T: CetkaikRepresentation>(
    cand: &PureMove_<T::AbsoluteCoord>,
    pure_game_state: &GroundState_<T>,
) -> Option<T::AbsoluteCoord> {
    match cand {
        PureMove_::NonTamMoveSrcDst { dest, .. } => {
            if T::as_board_absolute(&pure_game_state.f)
                .peek(*dest)
                .is_some()
            {
                Some(*dest)
            } else {
                None
            }
        }
        PureMove_::NonTamMoveSrcStepDstFinite { src, dest, .. }
        | PureMove_::InfAfterStep {
            src,
            planned_direction: dest,
            ..
        } => {
            // self-occlusion possible
            if src == dest {
                None
            } else if T::as_board_absolute(&pure_game_state.f)
                .peek(*dest)
                .is_some()
            {
                Some(*dest)
            } else {
                None
            }
        }
        PureMove_::NonTamMoveFromHopZuo { .. }
        | PureMove_::TamMoveNoStep { .. }
        | PureMove_::TamMoveStepsDuringFormer { .. }
        | PureMove_::TamMoveStepsDuringLatter { .. } => None,
    }
}

pub enum TacticsKey {
    VictoryAlmostCertain,
    StrengthenedShaman,
    FreeLunch,
    AvoidDefeat,
    LossAlmostCertain,
    Neutral,
}

impl std::fmt::Display for TacticsKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} / {}",
            match self {
                TacticsKey::VictoryAlmostCertain => "我須上行。",
                TacticsKey::StrengthenedShaman => "為激巫。",
                TacticsKey::FreeLunch => "我為無与之手。",
                TacticsKey::AvoidDefeat => "心為此而無行下行之道。",
                TacticsKey::LossAlmostCertain => "為何即下行。行我心之道。",
                TacticsKey::Neutral => "無心来為何善。行周時無下行之道。",
            },
            match self {
                TacticsKey::VictoryAlmostCertain => "勝ち確",
                TacticsKey::StrengthenedShaman => "激巫作成",
                TacticsKey::FreeLunch => "ただ取り",
                TacticsKey::AvoidDefeat => "負けを避けるためにこう指してみるか",
                TacticsKey::LossAlmostCertain => "なにやっても負けそうなので好き勝手に指す",
                TacticsKey::Neutral =>
                    "いい手が思いつかなかったので、即負けしない範囲で好き勝手に指す",
            }
        )
    }
}

fn is_tam_hue_absolute<T: CetkaikRepresentation>(
    coord: T::AbsoluteCoord,
    field: &T::AbsoluteField,
    tam_itself_is_tam_hue: bool,
) -> bool {
    let p = T::get_one_perspective();
    is_tam_hue_relative::<T>(
        T::to_relative_coord(coord, p),
        *T::as_board_relative(&T::to_relative_field(field.clone(), p)),
        tam_itself_is_tam_hue,
    )
}

fn every_luck_works<T>(p: Probabilistic<T>) -> T {
    match p {
        Probabilistic::Pure(k) => k,
        Probabilistic::Water { failure, success } => success,
        Probabilistic::Sticks {
            s0,
            s1,
            s2,
            s3,
            s4,
            s5,
        } => s5,
        Probabilistic::WhoGoesFirst { ia_first, a_first } => {
            panic!("WhoGoesFirst should not be given to `every_luck_works`")
        }
    }
}

pub fn apply_move_assuming_every_luck_works<
    T: CetkaikRepresentation + std::clone::Clone + std::fmt::Debug,
>(
    config: Config,
    cand: &PureMove_<T::AbsoluteCoord>,
    old_state: &GroundState_<T>,
) -> cetkaik_full_state_transition::state::HandResolved_<T> {
    let cand: PureMove__<T::AbsoluteCoord> = (*cand).into();
    match cand {
        PureMove__::NormalMove(msg) => {
            let state = every_luck_works(apply_normal_move(old_state, msg, config).unwrap());
            resolve(&state, config)
        }
        PureMove__::InfAfterStep(msg) => {
            let excited_state =
                every_luck_works(apply_inf_after_step(old_state, msg, config).unwrap());
            let hnr = every_luck_works(
                apply_after_half_acceptance(
                    &excited_state,
                    AfterHalfAcceptance_ {
                        dest: Some(msg.planned_direction),
                    },
                    config,
                )
                .unwrap(),
            );
            resolve(&hnr, config)
        }
    }
}

/// 取られづらい激巫が作られているかを確認
pub fn is_safe_gak_tuk_newly_generated<T: CetkaikRepresentation + Clone + std::fmt::Debug>(
    config: Config,
    cand: &PureMove_<T::AbsoluteCoord>,
    pure_game_state: &GroundState_<T>,
) -> bool {
    let tuk_coord = gak_tuk_newly_generated(cand, pure_game_state);
    let Some(tuk_coord) = tuk_coord else {
        return false;
    };

    let next: HandResolved_<T> =
        apply_move_assuming_every_luck_works(config, cand, pure_game_state);
    let next = match next {
        HandResolved_::NeitherTymokNorTaxot(k) => k,
        HandResolved_::HandExists { if_tymok, if_taxot } => if_tymok,
        HandResolved_::GameEndsWithoutTymokTaxot(_) => return false, // この場合はもうなんでもいいや
    };
    let candidates: Vec<PureMove_<T::AbsoluteCoord>> = not_from_hop1zuo1_candidates_vec::<T>(
        &cetkaik_yhuap_move_candidates::AllowKut2Tam2 {
            allow_kut2tam2: false,
        },
        config.tam_itself_is_tam_hue,
        pure_game_state.whose_turn,
        &next.f,
    );

    let countermeasures_exist = candidates.iter().any(|cand| {
        let cand = (*cand).into();
        // 行いづらい？
        if !is_likely_to_succeed::<T>(&cand) {
            return false;
        }

        // それは tuk_coord を侵害する？
        match cand {
            PureMove__::InfAfterStep(c) => c.planned_direction == tuk_coord,
            PureMove__::NormalMove(
                NormalMove_::NonTamMoveFromHopZuo { .. }
                | NormalMove_::TamMoveNoStep { .. }
                | NormalMove_::TamMoveStepsDuringFormer { .. }
                | NormalMove_::TamMoveStepsDuringLatter { .. },
            ) => false,
            PureMove__::NormalMove(
                NormalMove_::NonTamMoveSrcDst { dest, .. }
                | NormalMove_::NonTamMoveSrcStepDstFinite { dest, .. },
            ) => tuk_coord == dest,
        }
    });

    !countermeasures_exist
}

fn gak_tuk_newly_generated<T: CetkaikRepresentation>(
    cand: &PureMove_<T::AbsoluteCoord>,
    pure_game_state: &GroundState_<T>,
) -> Option<T::AbsoluteCoord> {
    let is_tam_hue = |dest| {
        is_tam_hue_absolute::<T>(
            dest,
            &pure_game_state.f,
            false, // don't care if tam itself is tam hue
        )
    };

    match cand {
        PureMove_::TamMoveNoStep { .. }
        | PureMove_::TamMoveStepsDuringFormer { .. }
        | PureMove_::TamMoveStepsDuringLatter { .. } => return None,
        PureMove_::NonTamMoveFromHopZuo { color, prof, dest } => {
            if *prof != Profession::Tuk2 {
                return None;
            }
            if is_tam_hue(*dest) {
                return Some(*dest);
            }
            None
        }
        PureMove_::NonTamMoveSrcDst { src, dest, .. }
        | PureMove_::NonTamMoveSrcStepDstFinite { src, dest, .. } => {
            let src_piece = T::as_board_absolute(&pure_game_state.f).peek(*src);
            let Some(src_piece) = src_piece else { return None; };
            src_piece.match_on_piece_and_apply(
                &|| panic!("Well, that should be TamMove"),
                &|color, prof, side| {
                    if prof != Profession::Tuk2 {
                        return None;
                    }
                    if
                    /* 結果として激巫が無い */
                    !is_tam_hue(*dest) || /* もとから激巫だった */ is_tam_hue(*src) {
                        return None;
                    }
                    Some(*dest)
                },
            )
        }
        PureMove_::InfAfterStep {
            src,
            step,
            planned_direction,
        } => {
            let src_piece = T::as_board_absolute(&pure_game_state.f)
                .peek(*src)
                .expect("No piece at src");
            src_piece.match_on_piece_and_apply(
                &|| panic!("Well, that should be TamMove"),
                &|color, prof, side| {
                    if prof != Profession::Tuk2 {
                        return None;
                    }
                    if
                    /* 結果として激巫が無い */
                    !is_tam_hue(*planned_direction) || /* もとから激巫だった */ is_tam_hue(*src)
                    {
                        return None;
                    }
                    Some(*planned_direction)
                },
            )
        }
    }
}

pub struct TacticsAndBotMove<Coord> {
    tactics: TacticsKey,
    bot_move: PureMove_<Coord>,
}

/// 0.「入水判定が必要であるか、4以上の踏越え判定が必要である」を「やりづらい(unlikely to succeed)」と定義する。
///    相手がある駒を取るのが「やりづらい」に相当する、若しくは不可能である、という場合、それを「取られづらい」と定義する。
///   「入水判定も要らず、2以下の踏越え判定しか要らない」を「やりやすい(very likely to succeed)」と定義する。
///
/// 強制発動戦略：
/// 1. 『無駄足は避けよ』：そもそもスタートとゴールが同一地点の手ってほぼ指さなくない？
/// 2. 『無駄踏みは避けよ』：踏まずに同じ目的地に行く手段があるなら、踏むな。
/// 3. 『勝ち確は行え』：駒を取って役が新たに完成し、その手がやりやすいなら、必ずそれを行う。
/// 4. 『負け確は避けよ』：取られづらくない駒で相手が役を作れて、それを避ける手があるなら、避ける手を指せ。一方で、「手を指した後で、取られづらくない駒で相手が役を作れる」もダメだなぁ。
/// 5. 『激巫は行え』：取られづらい激巫を作ることができるなら、常にせよ。
/// 6. 『ただ取りは行え』：駒を取ったとしてもそれがプレイヤーに取り返されづらい、かつ、その取る手そのものがやりづらくないなら、取る。
pub fn generate_move<T: CetkaikRepresentation + Clone + std::fmt::Debug>(
    rng: &mut SmallRng,
    config: Config,
    game_state: &GroundState_<T>,
    opponent_has_just_moved_tam: bool,
) -> TacticsAndBotMove<T::AbsoluteCoord> {
    let mut raw_candidates = not_from_hop1zuo1_candidates_vec::<T>(
        &cetkaik_yhuap_move_candidates::AllowKut2Tam2 {
            allow_kut2tam2: false,
        },
        config.tam_itself_is_tam_hue,
        game_state.whose_turn,
        &game_state.f,
    );
    raw_candidates.shuffle(rng);

    let candidates = raw_candidates
        .iter()
        .filter(|bot_cand| match bot_cand {
            PureMove_::TamMoveNoStep { .. }
            | PureMove_::TamMoveStepsDuringFormer { .. }
            | PureMove_::TamMoveStepsDuringLatter { .. } => {
                // 負け確回避とかなら読んでほしいので、候補に残す
                // ただし、opponent_has_just_moved_tam であるなら tam2 ty sak2 を防ぐべく除外する
                !opponent_has_just_moved_tam
            }
            PureMove_::InfAfterStep {
                src,
                step,
                planned_direction,
            } => {
                // 1. 『無駄足は避けよ』：そもそもスタートとゴールが同一地点の手ってほぼ指さなくない？
                if planned_direction == src {
                    return false;
                }

                // 2. 『無駄踏みは避けよ』：踏まずに同じ目的地に行く手段があるなら、踏むな。
                let better_option_exists = raw_candidates.iter().any(|c| {
                    match c {
                        // 有限で代用できるときも有限で代用しよう
                        PureMove_::NonTamMoveSrcDst {
                            src: src2, dest, ..
                        }
                        | PureMove_::NonTamMoveSrcStepDstFinite {
                            src: src2, dest, ..
                        } => src == src2 && planned_direction == dest,
                        PureMove_::InfAfterStep { .. }
                        | PureMove_::NonTamMoveFromHopZuo { .. }
                        | PureMove_::TamMoveNoStep { .. }
                        | PureMove_::TamMoveStepsDuringFormer { .. }
                        | PureMove_::TamMoveStepsDuringLatter { .. } => false,
                    }
                });
                if better_option_exists {
                    return false;
                }

                // 6マス以上飛ぶのは今回のルールでは無理です
                if T::absolute_distance(*planned_direction, *step) > 5 {
                    return false;
                }

                true
            }
            PureMove_::NonTamMoveFromHopZuo { .. } => {
                // 負け確回避とかなら読んでほしいので、除外しない
                true
            }
            PureMove_::NonTamMoveSrcDst { src, dest, .. } => {
                // 1. 『無駄足は避けよ』：そもそもスタートとゴールが同一地点の手ってほぼ指さなくない？
                if src == dest {
                    return false;
                }
                true
            }
            PureMove_::NonTamMoveSrcStepDstFinite { src, dest, .. } => {
                // 1. 『無駄足は避けよ』：そもそもスタートとゴールが同一地点の手ってほぼ指さなくない？
                if src == dest {
                    return false;
                }

                let better_option_exists = raw_candidates.iter().any(|c| match c {
                    PureMove_::NonTamMoveSrcDst {
                        src: src2,
                        dest: dest2,
                        ..
                    } => src == src2 && dest == dest2,
                    _ => false,
                });
                if better_option_exists {
                    return false;
                }
                true
            }
        })
        .collect::<Vec<_>>();

    let mut filtered_candidates = vec![];

    'bot_cand_loop: for bot_cand in &candidates {
        /****************
         *  強制発動戦略
         ****************/

        // 3. 『勝ち確は行え』：駒を取って役が新たに完成し、その手がやりやすいなら、必ずそれを行う。
        if is_victorious_hand(**bot_cand, game_state)
            && is_very_likely_to_succeed::<T>(&(**bot_cand).into())
        {
            return TacticsAndBotMove {
                tactics: TacticsKey::VictoryAlmostCertain,
                bot_move: **bot_cand,
            };
        }

        // 4. 『負け確は避けよ』：取られづらくない駒でプレイヤーが役を作れて、それを避ける手があるなら、避ける手を指せ。「手を指した後で、取られづらくない駒で相手が役を作れる」はダメだなぁ。

        //　in_danger: 避ける手を指せていたと仮定して、次の状態を呼び出し、
        // !in_danger: 次の状態を呼び出すと、今指したのが負けを確定させる手かどうかを調べることができる

        let next: HandResolved_<T> =
            apply_move_assuming_every_luck_works(config, bot_cand, game_state);
        let next = match next {
            HandResolved_::NeitherTymokNorTaxot(k) => k,
            HandResolved_::HandExists { if_tymok, if_taxot } => if_tymok,
            HandResolved_::GameEndsWithoutTymokTaxot(_) => panic!(), // この場合はもうなんでもいいや
        };

        let player_candidates: Vec<PureMove_<T::AbsoluteCoord>> =
            not_from_hop1zuo1_candidates_vec::<T>(
                &cetkaik_yhuap_move_candidates::AllowKut2Tam2 {
                    allow_kut2tam2: false,
                },
                config.tam_itself_is_tam_hue,
                next.whose_turn,
                &next.f,
            );

        for player_cand in player_candidates {
            if is_victorious_hand(player_cand, &next)
                && is_likely_to_succeed::<T>(&player_cand.into())
            {
                //  in_danger: 避ける手を指せていなかったことが判明した以上、この bot_cand を破棄して別の手を試してみる
                // !in_danger: 負けを確定させる手を指していた以上、この bot_cand を破棄して別の手を試してみる
                continue 'bot_cand_loop;
            }
        }

        // 5. 『激巫は行え』：取られづらい激巫を作ることができるなら、常にせよ。
        if is_safe_gak_tuk_newly_generated(config, bot_cand, game_state) {
            return TacticsAndBotMove {
                tactics: TacticsKey::StrengthenedShaman,
                bot_move: **bot_cand,
            };
        }
        // 6. 『ただ取りは行え』：駒を取ったとしてもそれがプレイヤーに取り返されづらい、かつ、その取る手そのものがやりづらくないなら、取る。

        let maybe_capture_coord = if_capture_get_coord(bot_cand, game_state);
        if let Some(capture_coord) = maybe_capture_coord {
            let next = apply_move_assuming_every_luck_works(config, bot_cand, game_state);
            let next = match next {
                HandResolved_::NeitherTymokNorTaxot(k) => k,
                HandResolved_::HandExists { if_tymok, if_taxot } => if_tymok,
                HandResolved_::GameEndsWithoutTymokTaxot(_) => panic!(), // この場合はもうなんでもいいや
            };
            let player_candidates = not_from_hop1zuo1_candidates_vec::<T>(
                &AllowKut2Tam2 {
                    allow_kut2tam2: false,
                },
                config.tam_itself_is_tam_hue,
                next.whose_turn,
                &next.f,
            );

            // 取り返すような手があるか？

            let take_back_exists = player_candidates.iter().any(|player_cand| {
                let capture_coord2 = if_capture_get_coord(player_cand, game_state);
                let Some(capture_coord2) = capture_coord2 else { return false; };
                if capture_coord == capture_coord2 {
                    // 取り返している
                    return true;
                }
                false
            });

            // 取り返せない、かつ、やりづらくない手であれば、指してみてもいいよね
            if !take_back_exists && is_likely_to_succeed::<T>(&(**bot_cand).into()) {
                return TacticsAndBotMove {
                    tactics: TacticsKey::FreeLunch,
                    bot_move: **bot_cand,
                };
            }
        }

        match bot_cand {
            PureMove_::NonTamMoveSrcDst { .. }
            | PureMove_::NonTamMoveSrcStepDstFinite { .. }
            | PureMove_::InfAfterStep { .. } => (),
            PureMove_::NonTamMoveFromHopZuo { .. }
            | PureMove_::TamMoveNoStep { .. }
            | PureMove_::TamMoveStepsDuringFormer { .. }
            | PureMove_::TamMoveStepsDuringLatter { .. } => continue, // まあ皇の動きは当分読まなくていいわ
                                                                      // まあ手駒を打つ手も当分読まなくていいわ
        }

        /*************************
         *  以上、強制発動戦略でした
         **************************/

        // 生き延びた候補を収容
        filtered_candidates.push(**bot_cand);
    }

    // 何やっても負け確、とかだと多分指す手がなくなるので、じゃあその時は好き勝手に指す
    if filtered_candidates.is_empty() {
        return TacticsAndBotMove {
            tactics: TacticsKey::LossAlmostCertain,
            bot_move: **candidates.choose(rng).unwrap(),
        };
    }

    let in_danger = (|| {
        let game_state_inverted = GroundState_ {
            whose_turn: !game_state.whose_turn,
            ..(*game_state).clone()
        };

        let candidates = not_from_hop1zuo1_candidates_vec::<T>(
            &cetkaik_yhuap_move_candidates::AllowKut2Tam2 {
                allow_kut2tam2: false,
            },
            config.tam_itself_is_tam_hue,
            game_state_inverted.whose_turn,
            &game_state_inverted.f,
        );

        for player_cand in candidates {
            if is_victorious_hand(player_cand, &game_state_inverted)
                && is_likely_to_succeed::<T>(&player_cand.into())
            {
                return true;
            }
        }
        false
    })();

    let bot_cand = filtered_candidates.choose(rng).unwrap();
    TacticsAndBotMove {
        tactics: if in_danger {
            TacticsKey::AvoidDefeat
        } else {
            TacticsKey::Neutral
        },
        bot_move: *bot_cand,
    }
}
