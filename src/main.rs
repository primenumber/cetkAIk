use cetkaik_full_state_transition::state::*;
use cetkaik_full_state_transition::message::*;
use cetkaik_full_state_transition::*;
use rand::prelude::*;
use rand::rngs::SmallRng;

struct HandExists {
    if_tymok: GroundState,
    if_taxot: IfTaxot,
}

#[derive(Clone)]
enum TymokOrTaxot {
    Tymok(GroundState),
    Taxot(IfTaxot),
}

trait CetkaikEngine {
    fn search(&mut self, s: &GroundState) -> Option<PureMove>;
    fn search_excited(&mut self, s: &ExcitedState) -> Option<AfterHalfAcceptance>;
    fn search_hand_resolved(&mut self, s: &HandExists) -> Option<TymokOrTaxot>;
}

struct RandomPlayer {
    config: Config,
    rng: SmallRng,
}

impl RandomPlayer {
    fn new(config: Config) -> RandomPlayer {
        RandomPlayer {
            config,
            rng: SmallRng::from_entropy(),
        }
    }
}

impl CetkaikEngine for RandomPlayer {
    fn search(&mut self, s: &GroundState) -> Option<PureMove> {
        let (hop1zuo1_candidates, candidates) = s.get_candidates(self.config);
        let pure_move_1 = hop1zuo1_candidates.choose(&mut self.rng);
        let pure_move_2 = candidates.choose(&mut self.rng);
        pure_move_1.or(pure_move_2).cloned()
    }

    fn search_excited(&mut self, s: &ExcitedState) -> Option<AfterHalfAcceptance> {
        let candidates = s.get_candidates(self.config);
        candidates.choose(&mut self.rng).copied()
    }

    fn search_hand_resolved(&mut self, s: &HandExists) -> Option<TymokOrTaxot> {
        [TymokOrTaxot::Tymok(s.if_tymok.clone()), TymokOrTaxot::Taxot(s.if_taxot.clone())].choose(&mut self.rng).cloned()
    }
}

fn main() {
    let config = Config::cerke_online_alpha();
    let pstate = initial_state();
    //eprintln!("{:?}", pstate);
    let (mut state, _) = pstate.choose();
    let mut searcher = RandomPlayer::new(config);
    loop {
        println!("{:?} {} {}", state.season, state.scores.ia(), state.scores.a());
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
