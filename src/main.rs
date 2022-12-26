mod cetkaik_engine;
mod greedy;
mod random_player;
/// cerke_online の CPU 対戦でいま使われている実装【神機】（「気まぐれな機械」）の移植
mod tun2_kik1;
use cetkaik_compact_representation::CetkaikCompact;
use cetkaik_engine::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_fundamental::AbsoluteSide::{ASide, IASide};
use cetkaik_fundamental::ColorAndProf;
use cetkaik_naive_representation::CetkaikNaive;
use cetkaik_render_to_console::*;
use cetkaik_traits::CetkaikRepresentation;
use greedy::*;
use random_player::*;

fn do_match<T: CetkaikRepresentation + Clone>(
    config: Config,
    ia_player: &mut dyn CetkaikEngine<T>,
    a_player: &mut dyn CetkaikEngine<T>,
    hide_move: bool,
    hide_board: bool,
    hide_ciurl: bool,
) -> (Victor, usize)
where
    T::AbsoluteField: PrintToConsole,
    T::AbsoluteCoord: std::fmt::Display,
{
    let mut state: GroundState_<T> = initial_state().choose().0;
    let mut turn_count = 0;
    loop {
        if !hide_board {
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
            panic!("No move possible");
        }
        let pure_move: PureMove__<T::AbsoluteCoord> = pure_move.unwrap();
        if !hide_move {
            println!("Move (Debug): {:?}", pure_move);
            println!("Move (Display): {}", pure_move);
        }
        let (hnr_state, water_entry_ciurl) = match pure_move {
            PureMove__::NormalMove(m) => apply_normal_move(&state, m, config).unwrap().choose(),
            PureMove__::InfAfterStep(m) => {
                let (ext_state, inf_after_step_ciurl) =
                    apply_inf_after_step::<T>(&state, m, config)
                        .unwrap()
                        .choose();
                if !hide_ciurl {
                    println!("InfAfterStep ciurl: {:?}", inf_after_step_ciurl)
                }
                let aha_move: AfterHalfAcceptance_<T::AbsoluteCoord> = searcher
                    .search_excited(&m, &ext_state, inf_after_step_ciurl)
                    .unwrap();
                if !hide_move {
                    println!("Move (excited) (Debug): {:?}", aha_move);
                    println!(
                        "Move (excited) (Display): {}",
                        aha_move.dest.map_or("None".to_string(), |c| c.to_string())
                    );
                }
                apply_after_half_acceptance(&ext_state, aha_move, config)
                    .unwrap()
                    .choose()
            }
        };
        if !hide_ciurl && water_entry_ciurl.is_some() {
            println!("water entry ciurl: {:?}", water_entry_ciurl)
        }
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
                                println!("Won: {:?}\nTotal turns: {}\n", v, turn_count);
                                return (v, turn_count);
                            }
                        }
                    }
                }
            }
            HandResolved_::GameEndsWithoutTymokTaxot(v) => {
                println!("Won: {:?}\nTotal turns: {}\n", v, turn_count);
                return (*v, turn_count);
            }
        }
        turn_count += 1;
    }
}

use clap::{Parser, ValueEnum};
use tun2_kik1::Tun2Kik1;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Don't show what move was played
    #[arg(long, default_value_t = false)]
    hide_move: bool,

    /// Don't print the board to console
    #[arg(long, default_value_t = false)]
    hide_board: bool,

    /// Don't print the result of ciurl
    #[arg(long, default_value_t = false)]
    hide_ciurl: bool,

    /// Don't print the AI's message
    #[arg(long, default_value_t = false)]
    hide_custom_message: bool,

    /// Only print the winner
    #[arg(long, default_value_t = false)]
    quiet: bool,

    /// How many matches to run
    #[arg(short, long, default_value_t = 1)]
    count: usize,

    /// The algorithm used by IASide
    #[arg(long, value_enum, default_value_t = Algorithm::Greedy)]
    ia_side: Algorithm,

    /// The algorithm used by ASide
    #[arg(long, value_enum, default_value_t = Algorithm::Greedy)]
    a_side: Algorithm,

    /// Internal implementation
    #[arg(long, value_enum, default_value_t = Implementation::Compact)]
    internal: Implementation,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Algorithm {
    Random,
    Greedy,
    Tunkik,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Implementation {
    Naive,
    Compact,
}

impl Algorithm {
    fn to_player<T: CetkaikRepresentation + Clone + std::fmt::Debug>(
        self,
        config: Config,
        hide_custom_message: bool,
    ) -> Box<dyn CetkaikEngine<T>> {
        match self {
            Algorithm::Random => Box::new(RandomPlayer::new(config)),
            Algorithm::Greedy => Box::new(GreedyPlayer::new(config)),
            Algorithm::Tunkik => Box::new(Tun2Kik1::new(config, !hide_custom_message)),
        }
    }
}

fn main() {
    use std::collections::HashMap;
    let config = Config::cerke_online_alpha();

    let args = Args::parse();

    let mut win_count: HashMap<Victor, usize> = HashMap::new();

    let mut turn_counts = vec![];

    for i in 0..args.count {
        println!("match #{}", i);
        let (victor, turn_count) = match args.internal {
            Implementation::Naive => do_match::<CetkaikNaive>(
                config,
                &mut *args
                    .ia_side
                    .to_player(config, args.quiet || args.hide_custom_message),
                &mut *args
                    .a_side
                    .to_player(config, args.quiet || args.hide_custom_message),
                args.quiet || args.hide_move,
                args.quiet || args.hide_board,
                args.quiet || args.hide_ciurl,
            ),
            Implementation::Compact => do_match::<CetkaikCompact>(
                config,
                &mut *args
                    .ia_side
                    .to_player(config, args.quiet || args.hide_custom_message),
                &mut *args
                    .a_side
                    .to_player(config, args.quiet || args.hide_custom_message),
                args.quiet || args.hide_move,
                args.quiet || args.hide_board,
                args.quiet || args.hide_ciurl,
            ),
        };
        turn_counts.push(turn_count);

        *win_count.entry(victor).or_insert(0) += 1;
    }

    println!(
        "Statistics:
ASide is {:?}, IASide is {:?}
Winner: {win_count:?}
average # of turns: {}
standard deviation of turns: {}
",
        args.a_side,
        args.ia_side,
        mean(&turn_counts).unwrap(),
        std_deviation(&turn_counts).unwrap(),
    );
}

fn mean(data: &[usize]) -> Option<f64> {
    let sum = data.iter().sum::<usize>() as f64;
    let len = data.len();

    match len {
        positive if positive > 0 => Some(sum / len as f64),
        _ => None,
    }
}

fn std_deviation(data: &[usize]) -> Option<f64> {
    match (mean(data), data.len()) {
        (Some(data_mean), len) if len > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f64);

                    diff * diff
                })
                .sum::<f64>()
                / len as f64;

            Some(variance.sqrt())
        }
        _ => None,
    }
}
