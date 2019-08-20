#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use clap::{App, ArgMatches, SubCommand, Arg};

use rustomata::automata::tree_stack_automaton::TreeStackAutomaton;
use rustomata::grammars::pmcfg::{PMCFG, PMCFGRule};
use log_domain::LogDomain;
use rustomata::approximation::equivalence_classes::EquivalenceRelation;
use rustomata::approximation::relabel::RlbElementTSA;
use rustomata::automata::tree_stack_automaton::PosState;
use rustomata::recognisable::Recognisable;
use crate::rustomata::approximation::ApproximationStrategy;
use rustomata::recognisable::Item;
use crate::rustomata::recognisable::automaton::Automaton;


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
    let accepting_string = vec!["a".to_string(), "a".to_string(), "b".to_string(), "c".to_string(), "c".to_string(), "d".to_string()];
    let not_accepting_string = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    let g: PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();
    let a = TreeStackAutomaton::from(g);

    let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = CLASSES_STRING.parse().unwrap();
    let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));
    let rlb = RlbElementTSA::new(&f);

    let (b, approx_inst) = rlb.approximate_automaton(&a);

    let a_acc = a.recognise(accepting_string.clone());
    let a_not_acc = a.recognise(not_accepting_string.clone());

    let b_acc = b.recognise(accepting_string.clone());
    let b_not_acc = b.recognise(not_accepting_string.clone());

    println!("------ automaton a ------");
    println!("{}", &a);
    println!();
    println!("------ automaton b ------");
    println!("{}", &b);
    println!();
    println!();

    println!("------ a accepting string ------");
    for Item(conf,_) in a_acc {
        println!("{}", conf);
        //println!("{}", b);
        println!();
    }
    println!();
    println!();

    println!("------ a not accepting string ------");
    for Item(conf,_) in a_not_acc {
        println!("{}", conf);
        println!();
    }
    println!();
    println!();
    
    println!("------ b accepting string ------");
    for Item(conf,run) in b_acc {
        println!("{}", &conf);
        let unapproxs = approx_inst.unapproximate_run(run);
        for unapprox in unapproxs {
            let checked = a.check_run(unapprox);
            for check in checked {
                println!("{:?}", check);
            }
        }
        println!();
    }
    println!();
    println!();
    
    println!("------ b not accepting string ------");
    for Item(conf,run) in b_not_acc {
        println!("{}", &conf);
        let unapproxs = approx_inst.unapproximate_run(run);
        for unapprox in unapproxs {
            let checked = a.check_run(unapprox);
            for check in checked {
                println!("{:?}", check);
            }
        }
        
        println!();
    }
    println!();
    println!();
    
}

