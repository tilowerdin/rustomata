#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use clap::{App, ArgMatches, SubCommand, Arg};
use rustomata::automata::tree_stack_automaton::TreeStackAutomaton;
use rustomata::grammars::pmcfg::{PMCFG,PMCFGRule};
use log_domain::LogDomain;
use rustomata::approximation::tts::TTSElement;
use rustomata::approximation::ApproximationStrategy;
use rustomata::approximation::equivalence_classes::EquivalenceRelation;
use rustomata::approximation::relabel::RlbElementTSA;
use rustomata::automata::tree_stack_automaton::PosState;
use rustomata::recognisable::Recognisable;
use rustomata::recognisable::Item;
use rustomata::recognisable::automaton::Automaton;


const GRAMMAR_STRING : &str = "
initial: [S]

S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)
A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5
A → [[],  []                             ] (    )   # 0.5
B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5
B → [[],  []                             ] (    )   # 0.5
";

const CLASSES_STRING : &str = "
S [\"S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\"]
A [\"A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\", \"A → [[],  []] (    )   # 0.5\"]
B [\"B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\", \"B → [[],  []] (    )   # 0.5\"]
R *
";

// const classes_string = "
// S [\"S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\"]
// A1 [\"A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\"]
// A2 [\"A → [[],  []                             ] (    )   # 0.5\"]
// B1 [\"B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\"]
// B2 [\"B → [[],  []                             ] (    )   # 0.5\"]
// R *
// ";

const AUTHOR : &str = "Tilo Werdin <tilo.werdin@tu-dresden.de>";


pub fn get_sub_command() -> App<'static, 'static> {
    // let poss_val = ["tts", "relabel", "ptk"] // ptk currently not available
    let poss_vals = ["tts", "relabel"];

    SubCommand::with_name("ctf-eval")
        .author(AUTHOR)
        .about("evaluation of the coarse to fine approach using different combinations")
        .subcommand(
            SubCommand::with_name("test")
        )
}

pub fn handle_sub_matches(ctf_matches: &ArgMatches) {

    match ctf_matches.subcommand() {
        ("test", _) => test(),
        _ => ()
    }

}

pub fn test() {
    let accepting_input = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];

    let g : PMCFG<String,String,LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

    let a = TreeStackAutomaton::from(g);

    println!("------ automaton a ------");
    println!("{}", &a);
    println!();
    println!();

    let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = CLASSES_STRING.parse().unwrap();
    let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));
    let rlb = RlbElementTSA::new(&f);

    let (b,strat1) = rlb.approximate_automaton(&a);

    println!("------ automaton b ------");
    println!("{}", &b);
    println!();
    println!();

    // for trans in c.transitions() {
    //     println!("------ unapproximate {} ------", &trans);
    //     for unapprox in strat2.unapproximate_transition(&trans) {
    //         println!("{}", unapprox);
    //     }
    //     println!();
    //     println!();
    // }



    let recs_b = b.recognise(accepting_input.clone());

    for Item(_,run_b) in recs_b {
        println!("------ run b ------");
        println!("{:?}", &run_b);
        println!();
        println!();

        let unapproxs1 = strat1.unapproximate_run(run_b);
        for unapprox1 in unapproxs1 {
            // println!("------ unapproximated run ------");
            // println!("{:?}", &unapprox2);
            // println!();
            // println!();
            let checked_as = a.check_run(unapprox1);
            for Item(_,checked_a) in checked_as {
                println!("------ run a ------");
                println!("{:?}", &checked_a);
                println!();
                println!();
            }
        }
    }

    let runs = a.recognise(accepting_input.clone());
    for Item(_,run) in runs {
        println!("------ orig run a ------");
        println!("{:?}", &run);
        println!();
        println!();
    }
    
}

