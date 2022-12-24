mod cetkaik_engine;
mod greedy;
mod random_player;
use cetkaik_compact_representation::CetkaikCompact;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_fundamental::AbsoluteSide::{ASide, IASide};
use cetkaik_fundamental::ColorAndProf;
use cetkaik_render_to_console::*;
use cetkaik_traits::CetkaikRepresentation;
use greedy::*;
//use random_player::*;
use cetkaik_engine::*;

fn do_match<T: CetkaikRepresentation + Clone>(
    config: Config,
    ia_player: &mut dyn CetkaikEngine<T>,
    a_player: &mut dyn CetkaikEngine<T>,
    hide_move: bool,
    log_board: bool,
) where
    T::AbsoluteField: PrintToConsole,
    T::AbsoluteCoord: std::fmt::Display,
{
    let mut state: GroundState_<T> = initial_state().choose().0;
    let mut turn_count = 0;
    loop {
        if log_board {
            println!("\n======================================");
            state.f.print_to_console();
        }
        if !hide_move {
            fn to_s(v: &[ColorAndProf]) -> String {
                v.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            }

            println!(
                "{}, Turn: {:?}, Season: {:?}, Scores: (IA:{}, A:{}), hop1zuo1: (IA: {}, A: {})",
                turn_count,
                state.whose_turn,
                state.season,
                state.scores.ia(),
                state.scores.a(),
                to_s(&T::hop1zuo1_of(IASide, &state.f)),
                to_s(&T::hop1zuo1_of(ASide, &state.f)),
            );
        }
        let searcher: &mut dyn CetkaikEngine<T> = match state.whose_turn {
            IASide => ia_player,
            ASide => a_player,
        };
        let pure_move = searcher.search(&state);
        if pure_move.is_none() {
            break;
        }
        let pure_move: PureMove__<T::AbsoluteCoord> = pure_move.unwrap();
        if !hide_move {
            println!("Move (Debug): {:?}", pure_move);
            println!("Move (Display): {}", pure_move);
        }
        let hnr_state = match pure_move {
            PureMove__::NormalMove(m) => apply_normal_move(&state, m, config).unwrap().choose().0,
            PureMove__::InfAfterStep(m) => {
                let ext_state = apply_inf_after_step::<T>(&state, m, config)
                    .unwrap()
                    .choose()
                    .0;
                let aha_move = searcher.search_excited(&m, &ext_state).unwrap();
                if !hide_move {
                    println!("Move(excited): {:?}", aha_move);
                }
                apply_after_half_acceptance(&ext_state, aha_move, config)
                    .unwrap()
                    .choose()
                    .0
            }
        };
        let resolved = resolve(&hnr_state, config);
        match &resolved {
            HandResolved_::NeitherTymokNorTaxot(s) => state = s.clone(),
            HandResolved_::HandExists { if_tymok, if_taxot } => {
                let he = HandExists_ {
                    if_tymok: if_tymok.clone(),
                    if_taxot: if_taxot.clone(),
                };
                match searcher.search_hand_resolved(&he).unwrap() {
                    TymokOrTaxot_::Tymok(s) => state = s,
                    TymokOrTaxot_::Taxot(t) => {
                        if !hide_move {
                            println!("Taxot!");
                        }
                        match t {
                            IfTaxot_::NextSeason(ps) => state = ps.clone().choose().0,
                            IfTaxot_::VictoriousSide(v) => {
                                println!("Won: {:?}", v);
                                break;
                            }
                        }
                    }
                }
            }
            HandResolved_::GameEndsWithoutTymokTaxot(v) => {
                println!("Won: {:?}", v);
                break;
            }
        }
        turn_count += 1;
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Don't show what move was played
    #[arg(long, default_value_t = true)]
    hide_move: bool,

    /// Don't print the board to console
    #[arg(long, default_value_t = true)]
    hide_board: bool,

    /// How many matches to run
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let config = Config::cerke_online_alpha();

    let args = Args::parse();

    for _ in 0..args.count {
        // ここを CetkaikCore にすると古い遅い実装が走る
        do_match::<CetkaikCompact>(
            config,
            &mut GreedyPlayer::new(config),
            &mut GreedyPlayer::new(config),
            args.hide_move,
            !args.hide_board,
        );
    }
}
