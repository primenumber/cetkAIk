mod cetkaik_engine;
mod random_player;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;
use cetkaik_core::absolute::Side;
use random_player::*;
use cetkaik_engine::*;

fn main() {
    let config = Config::cerke_online_alpha();
    let pstate = initial_state();
    //eprintln!("{:?}", pstate);
    let (mut state, _) = pstate.choose();
    let mut ia_searcher = RandomPlayer::new(config);
    let mut a_searcher = RandomPlayer::new(config);
    loop {
        println!("{:?} {} {}", state.season, state.scores.ia(), state.scores.a());
        let searcher = match state.whose_turn {
            Side::IASide => &mut ia_searcher,
            Side::ASide => &mut a_searcher,
        };
        let pure_move = searcher.search(&state);
        if pure_move.is_none() {
            break;
        }
        let pure_move = pure_move.unwrap();
        println!("{:?}", pure_move);
        let hnr_state = match pure_move {
            PureMove::NormalMove(m) => {
                 apply_normal_move(&state, m, config).unwrap().choose().0
            },
            PureMove::InfAfterStep(m) => {
                let ext_state = apply_inf_after_step(&state, m, config).unwrap().choose().0;
                let aha_move = searcher.search_excited(&ext_state).unwrap();
                println!("{:?}", aha_move);
                apply_after_half_acceptance(&ext_state, aha_move, config).unwrap().choose().0
            }
        };
        let resolved = resolve(&hnr_state, config);
        match &resolved {
            HandResolved::NeitherTymokNorTaxot(s) => state = s.clone(),
            HandResolved::HandExists{if_tymok, if_taxot} => {
                let he = HandExists {
                    if_tymok: if_tymok.clone(),
                    if_taxot: if_taxot.clone(),
                };
                match searcher.search_hand_resolved(&he).unwrap() {
                    TymokOrTaxot::Tymok(s) => state = s,
                    TymokOrTaxot::Taxot(t) => {
                        println!("Taxot!");
                        match t {
                            IfTaxot::NextSeason(ps) => state = ps.clone().choose().0,
                            IfTaxot::VictoriousSide(v) => {
                                println!("Won: {:?}", v);
                                break;
                            },
                        }
                    }
                }
            },
            HandResolved::GameEndsWithoutTymokTaxot(v) => {
                println!("Won: {:?}", v);
                break;
            },
        }
    }
}
