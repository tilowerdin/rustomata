#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use clap::{App, ArgMatches, SubCommand, Arg};


use std::fs::File;
use std::io::Read;
use std::io;
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
use rustomata::approximation::relabel::{RlbElement,RlbElementTSA};
use std::str::FromStr;

use rustomata::grammars::cfg::CFG;
use crate::rustomata::recognisable::automaton::Automaton;

use std::fmt::{Debug,Display};

use std::cmp::min;

use std::ops::MulAssign;
use num_traits::identities::One;




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
            SubCommand::with_name("cfg")
                .author(AUTHOR)
                .arg(
                    Arg::with_name("grammar")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("rlb")
                        .long("rlb")
                        .multiple(true)
                        .takes_value(true)
                        .number_of_values(1)
                )
                .arg(
                    Arg::with_name("ptk")
                        .long("ptk")
                        .multiple(true)
                        .takes_value(true)
                        .number_of_values(1)
                )
        )
        .subcommand(
            SubCommand::with_name("mcfg")
                .author(AUTHOR)
                .arg(
                    Arg::with_name("grammar")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("tts")
                        .long("tts")
                )
                .arg(
                    Arg::with_name("rlb")
                        .long("rlb")
                        .multiple(true)
                        .takes_value(true)
                        .number_of_values(1)
                )
                .arg(
                    Arg::with_name("ptk")
                        .long("ptk")
                        .multiple(true)
                        .takes_value(true)
                        .number_of_values(1)
                )
        )
        .subcommand(
            SubCommand::with_name("test")
        )
}

pub fn handle_sub_matches(ctf_matches: &ArgMatches) {

    match ctf_matches.subcommand() {
        ("cfg", Some(cfg_matches)) => {
            handle_cfg_matches(&cfg_matches);
        }
        ("mcfg", Some(mcfg_matches)) => {
            handle_mcfg_matches(&mcfg_matches);
        },
        ("test", _) => test(),
        _ => ()
    }

}

macro_rules! recognise {
    ( $recogniser:expr ) => {
        {
            let mut corpus = String::new();
            io::stdin().read_to_string(&mut corpus);
            for line in corpus.lines() {
                let word = line.split_whitespace().map(|x| x.to_string()).collect();
                let parses = $recogniser.recognise(word);

                for Item(conf,parse) in parses {
                    println!("{:?}", conf);
                    println!("{:?}", parse);
                }
            }
        }
    }
}

// handle a given cfg grammar with a pda
pub fn handle_cfg_matches(cfg_matches : &ArgMatches) {
    println!("cfg\n");

    println!("{:#?}", cfg_matches);
}

// handle a given mcfg grammar with a tsa
pub fn handle_mcfg_matches(mcfg_matches : &ArgMatches) {
    println!("mcfg\n");



    // TODO
    let g : PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

    let a = TreeStackAutomaton::from(g);

    let mut approx_matches = get_approx_args(mcfg_matches);
    
    
    let tts_string = "tts".to_string();
    let ptk_string = "ptk".to_string();
    let rlb_string = "rlb".to_string();


    // TODO: this is not very elegant and it is only possible to parse 
    // explicitly programmed combinations of approximation strategies
    // the problem is that the strategies do not have the same type and
    // it is impossible to create a list of Box<dyn ApproximationStrategy<...>>
    // since ApproximationStrategy itself has generic parameters
    // currently possible:
    //      - tts
    //      - tts, rlb
    //      - rlb
    //      - rlb, tts
    // currently developing:
    //      - rlb, tts, rlb
    // match first strategy
    match approx_matches.pop() {
        Some((first_strategy, fst_additional)) => {
            if first_strategy == tts_string {
                
                // create the tts strategy
                let s1 = TTSElement::new();

                // match the second strategy having tts as first strategy
                match approx_matches.pop() {
                    // no second strategy
                    // use original macro to create the CoarseToFineRecogniser
                    None => {
                        let recogniser = coarse_to_fine_recogniser!(a; s1);
                        recognise!(recogniser);
                    },

                    Some((second_strategy, sec_additional)) => {
                        if second_strategy == rlb_string {
                            let rlb_file = sec_additional.unwrap();
                            // create the rlb strategy
                            let classes_string = read_file(rlb_file);
                            let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = classes_string.parse().unwrap();
                            let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));
                            let s2 = RlbElement::new(&f);

                            // match the third strategy having tts, rlb
                            match approx_matches.pop() {
                                // no third strategy
                                // use original macro to create the CoarseToFineRecogniser
                                None => {
                                    let recogniser = coarse_to_fine_recogniser!(a; s1, s2);
                                    recognise!(recogniser);
                                },

                                Some((third_strategy, trd_additional)) => {
                                    if third_strategy == ptk_string {
                                        panic!("ptk is currently not implemented!");
                                    } else {
                                        panic!("{} not allowed here", third_strategy);
                                    }
                                },
                            }
                        } else if second_strategy == ptk_string {
                            panic!("ptk is currently not implemented!");
                        } else {
                            panic!("{} not allowed here", second_strategy);
                        }
                    }
                }
            } else if first_strategy == rlb_string {
                let rlb_file = fst_additional.unwrap();

                // create the rlb strategy
                let classes_string = read_file(rlb_file);
                let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = classes_string.parse().unwrap();
                let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));
                let s1 = RlbElementTSA::new(&f);

                // match the second strategy having rlb as first strategy
                match approx_matches.pop() {
                    // no second strategy 
                    // use original macro to create CoarseToFineRecogniser
                    None => {
                        let recogniser = coarse_to_fine_recogniser!(a; s1);
                        recognise!(recogniser);
                    },

                    Some((second_strategy, sec_additional)) => {
                        if second_strategy == tts_string {
                            let s2 = TTSElement::new();

                            // match the third strategy having rlb, tts
                            match approx_matches.pop() {
                                // no third strategy 
                                // use original macro to create CoarseToFineRecogniser
                                None => {
                                    let recogniser = coarse_to_fine_recogniser!(a; s1, s2);
                                    recognise!(recogniser);
                                },

                                Some((third_strategy, trd_additional)) => {
                                    if third_strategy == ptk_string {
                                        panic!("ptk is currently not implemented!");
                                    } else {
                                        panic!("{} not allowed here", third_strategy);
                                    }
                                },
                            }
                        } else {
                            panic!("{} not allowed here", second_strategy);
                        }
                    },
                }
            } else {
                panic!("{} not allowed here", first_strategy);
            }
        },
        _ => panic!("Not implemented yet!"),
    };
    
}



// search for the approximation arguments and sort them in a list
// with their parameters if present
// searching for tts, rlb and ptk
pub fn get_approx_args(arg_matches : &ArgMatches) -> Vec<(String, Option<String>)> {
    let mut vec = Vec::new();
    
    // tts
    vec.append(&mut get_tuple_vec(arg_matches, "tts"));

    // rlb
    vec.append(&mut get_tuple_vec(arg_matches, "rlb"));

    // ptk
    vec.append(&mut get_tuple_vec(arg_matches, "ptk"));

    vec.sort();
    vec.iter().map(|(_,a,b)| (a.clone(),b.clone())).collect()
}

// return a list of tuples containing the indices, the given name
// and optinal arguments if present
fn get_tuple_vec(arg_matches : &ArgMatches, s : &str) 
-> Vec<(usize, String, Option<String>)> 
{
    let mut vec = Vec::new();
    // get indices
    if let Some(indices) = arg_matches.indices_of(s) {
        // get arguments
        if let Some(values) = arg_matches.values_of(s) {
            // compare lists and decide if arguments are present
            if indices.len() == values.len() {
                // if present, zip lists and push them 
                // into the resulting list
                let zipped = indices.zip(values);
                for (i,v) in zipped {
                    vec.push((i,s.to_string(),Some(v.to_string())));
                }
            } else {
                // if not present, push indices with None into list
                for i in indices {
                    vec.push((i,s.to_string(),None));
                }
            }
        } 
    }
    vec
}

// returns the content of the file given by the `path` string
fn read_file(path: String) -> String {
    let mut file = File::open(path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string);
    string
}

pub fn handle_sub_matches1(ctf_matches: &ArgMatches) {

    // parsen der MCFG
    let g: PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

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

    let rel : EquivalenceRelation<PMCFGRule<_,_,_>,String> 
        = EquivalenceRelation::from_str(CLASSES_STRING).unwrap();
    let mapping = |ps: &PosState<_>| ps.map(|nt| rel.project(nt));
    let rlb = RlbElement::new(&mapping);

    // let rec = coarse_to_fine_recogniser!(a;
    //     tts, 
    //     rlb);


    let (aut0, strat_inst0) 
    // :
    // (
    //     PushDownAutomaton<
    //         PosState<
    //             PMCFGRule<
    //                 String, 
    //                 String, 
    //                 LogDomain<f64>
    //             >
    //         >, 
    //         String, 
    //         LogDomain<f64>
    //     >
    //     , _
    // ) 
    = tts.approximate_automaton(&a);

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
                let unapp_transs = strat_inst1.unapproximate_transition(trans);
                for unapp_trans in unapp_transs {
                    print("unapproximated transs", &unapp_trans);
                }
                
            }



            let r0s = strat_inst1.unapproximate_run(pd);
            for r0 in r0s {
                print_debug("unapproximated run r0", &r0);
                
                let checked_runs0 = aut0.check_run(r0);
                for Item(cr0conf, cr0pd) in checked_runs0 {
                    print("checkedRun conf", &cr0conf);
                    print_debug("checkedRun pd", &cr0pd);

                    let rs = strat_inst0.unapproximate_run(cr0pd);
                    for r in rs {
                        print_debug("unapproximated run r", &r);

                        let checked_runs = a.check_run(r);
                        for Item(crconf, crts) in checked_runs {
                            print("checkedRun conf", &crconf);
                            print_debug("checkedRun ts", &crts);
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

fn print_debug<T>(title: &str, anything: &T)
where
    T: Debug
{
    println!("---------- {} ----------", title);
    println!("{:?}", anything);
    println!();
}

#[cfg(test)]
mod test {
    #[test]
    fn zip_test() {
        let vec1 = vec![1,2,3];
        let vec2 = vec!["a","b","c"];

        let zipped = super::zip(&vec1, &vec2);
        assert_eq!(zipped, vec![(1,"a"), (2,"b"), (3,"c")]);
    }

    #[test]
    fn zip_ord_test() {
        let vec1 = vec![2,7,3];
        let vec2 = vec!["a","b","c"];

        let solution : Vec<_> = vec![(2,"a"), (3,"c"), (7,"b")];

        let mut sorted = super::zip(&vec1, &vec2);
        sorted.sort();
        // let sorted = zipped.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(sorted, solution);
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


