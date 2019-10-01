#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use clap::{App, ArgMatches, SubCommand, Arg};

use rustomata::grammars::cfg::CFG;
use log_domain::LogDomain;
use rustomata::automata::push_down_automaton::{PushDown, PushDownAutomaton, PushDownInstruction};
use rustomata::approximation::ptk::PDTopKElement;
use crate::rustomata::approximation::ApproximationStrategy;
use rustomata::recognisable::Recognisable;
use rustomata::recognisable::Item;
use rustomata::recognisable::coarse_to_fine::CoarseToFineRecogniser;
use std::rc::Rc;

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

const CFG_STRING : &str = "
initial: [S]

S → [Nt A, Nt A, Nt A, Nt A, Nt A ] # 1
A → [T a                          ] # 0.6
A → [T b                          ] # 0.4
";

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
    let g : CFG<String, String, LogDomain<f64>> = CFG_STRING.parse().unwrap();
    let a = PushDownAutomaton::from(g);

    println!();
    println!("{}", a);
    println!();

    let ptk = PDTopKElement::new(5);
    
    // let rec = coarse_to_fine_recogniser!(a; ptk);

    // for Item(i1,i2) in rec.recognise(vec!["a".to_string(); 5]) {
    //     println!("{}", i1);
    //     println!("{:?}", i2);
    // }




    let (a1,_) = ptk.approximate_automaton(&a);

    println!();
    println!("{}", a1);
    println!();




    for Item(i1,i2) in a1.recognise(vec!["a".to_string(); 4]) {
        println!("{}", i1);
        println!("{:?}", i2);
    }
}

