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
use rustomata::recognisable::{Recognisable,Item};

use rustomata::automata::push_down_automaton::{PushState, PushDownAutomaton};
use rustomata::approximation::equivalence_classes::EquivalenceRelation;
use rustomata::approximation::relabel::RlbElement;
use std::str::FromStr;

use rustomata::grammars::cfg::CFG;
use crate::rustomata::recognisable::automaton::Automaton;

use std::fmt::{Debug,Display};

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

    // erstellen der MCFG als String
    let mut grammar_string = "
initial: [S]

S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)
A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5
A → [[],  []                             ] (    )   # 0.5
B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5
B → [[],  []                             ] (    )   # 0.5
";

    // erstellen von Äquivalenzklassen für relabel
//     let mut classes_string = "
// S [\"S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\"]
// A1 [\"A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\"]
// A2 [\"A → [[],  []                             ] (    )   # 0.5\"]
// B1 [\"B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\"]
// B2 [\"B → [[],  []                             ] (    )   # 0.5\"]
// R *
// ";

    let mut classes_string = "
S [\"S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\"]
A [\"A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\", \"A → [[],  []                             ] (    )   # 0.5\"]
B [\"B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\", \"B → [[],  []                             ] (    )   # 0.5\"]
R *
";

    // parsen der MCFG
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

    // print("TreeStackAutomaton a", &a);
    
    let tts = TTSElement::new();

    let rel : EquivalenceRelation<PMCFGRule<_,_,_>,String> = EquivalenceRelation::from_str(classes_string).unwrap();
    let mapping = |ps: &PosState<_>| ps.map(|nt| rel.project(nt));
    let rlb = RlbElement::new(&mapping);

    // let rec = coarse_to_fine_recogniser!(a;
    //     tts, 
    //     rlb);


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

    // print("approximated automaton aut0", &aut0);
    

    let (aut1, strat_inst1) = rlb.approximate_automaton(&aut0);
    print("approximated automaton aut1", &aut1);

    for key in strat_inst1.reverse_transition_map.keys() {
        println!("---------- {} ----------", &key);
        if let Some(list) = strat_inst1.reverse_transition_map.get(key) {
            for l in list {
                println!("{}", l);
            }
        }
        println!();
    }
    

    let corpus = String::from("a c\n");
    // let n = 1;
    for sentence in corpus.lines() {
        let word :Vec<_>= sentence.split_whitespace().map(|x| x.to_string()).collect();
        let word1 = word.clone();
        let word2 = word.clone();

        let erga = a.recognise(word);
        // for Item(conf,ts) in erga {
        //     print("a.recognise(word) conf", &conf);
        //     printDebug("a.recognise(word) ts", &ts);
        // }

        let ergaut0 = aut0.recognise(word1);
        // for Item(conf,pd) in ergaut0 {
        //     print("aut0.recognise(word) conf", &conf);
        //     printDebug("aut0.recognise(word) pd", &pd);
        // }

        let ergaut1 = aut1.recognise(word2);
        for Item(conf,pd) in ergaut1 {
            // print("aut1.recognise(word) conf", &conf);
            // printDebug("aut1.recognise(word) pd", &pd);

            let transs : Vec<_> = pd.clone().into();
            for trans in &transs {
                print("Transition of pd", trans);
                let unappTranss = strat_inst1.unapproximate_transition(trans);
                for unappTrans in unappTranss {
                    print("unapproximated transs", &unappTrans);
                }
                
            }



            let r0s = strat_inst1.unapproximate_run(pd);
            for r0 in r0s {
                printDebug("unapproximated run r0", &r0);
                
                let checkedRuns0 = aut0.check_run(r0);
                for Item(cr0conf, cr0pd) in checkedRuns0 {
                    print("checkedRun conf", &cr0conf);
                    printDebug("checkedRun pd", &cr0pd);

                    let rs = strat_inst0.unapproximate_run(cr0pd);
                    for r in rs {
                        printDebug("unapproximated run r", &r);

                        let checkedRuns = a.check_run(r);
                        for Item(crconf, crts) in checkedRuns {
                            print("checkedRun conf", &crconf);
                            printDebug("checkedRun ts", &crts);
                        }
                    }
                }
            }
        }
        

        // let mut forest0 : i32 = aut1.recognise(word);

        // let mut forest : Vec<_> = rec.recognise(word).collect();
        
        // for parse in &forest {
        //     println!("{:?}", parse);
        // }
        // println!();

    }
}

fn print<T>(title: &str, anything: &T)
where
    T: Display
{
    println!("---------- {} ----------", title);
    println!("{}", anything);
    println!();
}

fn printDebug<T>(title: &str, anything: &T)
where
    T: Debug
{
    println!("---------- {} ----------", title);
    println!("{:?}", anything);
    println!();
}