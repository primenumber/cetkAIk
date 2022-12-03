mod cetkaik_engine;
mod random_player;
mod greedy;
use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;
use cetkaik_core::absolute::Side;
use greedy::*;
use random_player::*;
use cetkaik_engine::*;

fn do_match(config: Config, ia_player: &mut dyn CetkaikEngine, a_player: &mut dyn CetkaikEngine) {
    let mut state = initial_state().choose().0;
    let mut turn_count = 0;
    loop {
        println!("Turn: {}, Season: {:?}, Scores: (IA:{}, A:{})", turn_count, state.season, state.scores.ia(), state.scores.a());
        let searcher: &mut dyn CetkaikEngine = match state.whose_turn {
            Side::IASide => ia_player,
            Side::ASide => a_player,
        };
        let pure_move = searcher.search(&state);
        if pure_move.is_none() {
            break;
        }
        let pure_move = pure_move.unwrap();
        println!("Move: {:?}", pure_move);
        let hnr_state = match pure_move {
            PureMove::NormalMove(m) => {
                 apply_normal_move(&state, m, config).unwrap().choose().0
            },
            PureMove::InfAfterStep(m) => {
                let ext_state = apply_inf_after_step(&state, m, config).unwrap().choose().0;
                let aha_move = searcher.search_excited(&ext_state).unwrap();
                println!("Move(excited): {:?}", aha_move);
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
        turn_count += 1;
    }
}

fn main() {
    let config = Config::cerke_online_alpha();
    let mut ia_searcher = RandomPlayer::new(config);
    let mut a_searcher = GreedyPlayer::new(config);
    do_match(config, &mut ia_searcher, &mut a_searcher);
}
