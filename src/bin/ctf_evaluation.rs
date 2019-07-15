#![allow(unused_imports)]

use clap::{App, ArgMatches, SubCommand, Arg};

use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use log_domain::LogDomain;

use rustomata::automata::tree_stack_automaton::{TreeStackAutomaton,PosState};
use rustomata::grammars::pmcfg::{PMCFG,PMCFGRule};
use rustomata::approximation::tts::TTSElement;
use rustomata::approximation::ApproximationStrategy;
use rustomata::recognisable::coarse_to_fine::CoarseToFineRecogniser;
use rustomata::recognisable::Recognisable;

use rustomata::automata::push_down_automaton::{PushState, PushDownAutomaton};
use rustomata::approximation::equivalence_classes::EquivalenceRelation;
use rustomata::approximation::relabel::RlbElement;
use std::str::FromStr;

use rustomata::grammars::cfg::CFG;



pub fn get_sub_command() -> App<'static, 'static> {
    // let poss_val = ["tts", "relabel", "ptk"] // ptk currently not available
    let poss_vals = ["tts", "relabel"];

    SubCommand::with_name("ctf-eval")
        .author("Tilo Werdin <tilo.werdin@tu-dresden.de>")
        .about("evaluation of the coarse to fine approach using different combinations")
        // .arg(Arg::with_name("strategy")
        //     .short("s")
        //     .long("strategy")
        //     .takes_value(true)
        //     .possible_values(&poss_vals)
        //     .min_values(1))
        // .arg(Arg::with_name("grammar")
        //     .required(true))
        // .arg(Arg::with_name("classes")
        //     .required(true))
        
}

pub fn handle_sub_matches(ctf_matches: &ArgMatches) {

    let mut grammar_string = "
initial: [S]

S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)
A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5
A → [[],  []                             ] (    )   # 0.5
B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5
B → [[],  []                             ] (    )   # 0.5
";

    let mut classes_string = "
S [S    ]
N [A, B ]
R *
";

    let g: PMCFG<String, String, LogDomain<f64>> = grammar_string.parse().unwrap();
    let a  : 
    TreeStackAutomaton<
        PosState<
            PMCFGRule<
                String, 
                String, 
                LogDomain<f64>
            >
        >, 
        String, 
        LogDomain<f64>
    > = TreeStackAutomaton::from(g);

    let tts = TTSElement::new();

    let rel : EquivalenceRelation<String,String> = EquivalenceRelation::from_str(classes_string).unwrap();
    let mapping = |ps: &PushState<_, _>| ps.map(|nt| rel.project(nt));
    let rlb = RlbElement::new(&mapping);

    let (aut0, strat_inst0) :
    (
        PushDownAutomaton<
            PosState<
                PMCFGRule<
                    String, 
                    String, 
                    LogDomain<f64>
                >
            >, 
            String, 
            LogDomain<f64>
        >
        , _
    ) = tts.approximate_automaton(&a);

    let (aut1, strat_inst1) = rlb.approximate_automaton(&aut0);


    // let rec = coarse_to_fine_recogniser!(a;
    //     tts, 
    //     rlb);

    let corpus = String::from("a c\n");
    // let n = 1;
    for sentence in corpus.lines() {
        let word :Vec<_>= sentence.split_whitespace().map(|x| x.to_string()).collect();

        let mut forest0 : i32 = aut1.recognise(word);

        // let mut forest : Vec<_> = rec.recognise(word).collect();
        
        // for parse in &forest {
        //     println!("{:?}", parse);
        // }
        // println!();

    }
}

