mod cetkaik_engine;
mod greedy;
mod random_player;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::*;
use cetkaik_fundamental::AbsoluteSide::{ASide, IASide};
use cetkaik_fundamental::ColorAndProf;

use cetkaik_compact_representation::CetkaikCompact;
use cetkaik_traits::CetkaikRepresentation;
use greedy::*;
//use random_player::*;
use cetkaik_engine::*;

fn do_match<T: CetkaikRepresentation + Clone>(
    config: Config,
    ia_player: &mut dyn CetkaikEngine<T>,
    a_player: &mut dyn CetkaikEngine<T>,
    hide_move: bool,
) {
    let mut state = initial_state().choose().0;
    let mut turn_count = 0;
    loop {
        if !hide_move {
            fn to_s(v: &[ColorAndProf]) -> String {
                let mut s = String::new();
                for (idx, &e) in v.iter().enumerate() {
                    if idx > 0 {
                        s += " ";
                    }
                    s += &format!("{}", e);
                }
                s
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
        let pure_move = pure_move.unwrap();
        if !hide_move {
            println!("Move: {:?}", pure_move);
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
    // Name of the person to greet
    // #[arg(short, long)]
    // name: String,

    /// Whether to show what hand was played
    #[arg(short, long, default_value_t = true)]
    log_move: bool,

    /// How many matches to run
    #[arg(short, long, default_value_t = 10)]
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
            !args.log_move,
        );
    }
}
