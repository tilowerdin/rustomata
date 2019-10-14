#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use clap::{App, ArgMatches, SubCommand, Arg};


use rustomata::approximation::ptk::PDTopKElement;
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

use rustomata::automata::push_down_automaton::{PushState, PushDownAutomaton, PushDown, PushDownInstruction};
use rustomata::approximation::equivalence_classes::EquivalenceRelation;
use rustomata::approximation::relabel::{RlbElement,RlbElementTSA};
use std::str::FromStr;

use rustomata::grammars::cfg::CFG;
use crate::rustomata::recognisable::automaton::Automaton;

use std::fmt::{Debug,Display};

use std::cmp::min;

use std::ops::MulAssign;
use num_traits::identities::One;
use rustomata::approximation::benchmarks;




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

const CLASSES_STRING2 : &str ="
A *
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
            SubCommand::with_name("bench_mcfg")
                .author(AUTHOR)
                .arg(
                    Arg::with_name("file_name")
                        .required(true)
                )
        )
}

pub fn handle_sub_matches(ctf_matches: &ArgMatches) {

    match ctf_matches.subcommand() {
        ("cfg", Some(cfg_matches)) => {
            handle_cfg_matches(&cfg_matches);
        },
        ("mcfg", Some(mcfg_matches)) => {
            handle_mcfg_matches(&mcfg_matches);
        },
        ("bench_mcfg", Some(mcfg_matches)) => bench_mcfg(&mcfg_matches),
        _ => ()
    }

}

macro_rules! recognise {
    ( $recogniser:expr ) => {
        {
            let mut corpus = String::new();
            let _ = io::stdin().read_to_string(&mut corpus);
            for line in corpus.lines() {
                let word = line.split_whitespace().map(|x| x.to_string()).collect();
                let parses = $recogniser.recognise(word);

                for Item(conf,parse) in parses {
                    println!("{}", conf);
                }
            }
        }
    }
}

// handle a given cfg grammar with a pda
pub fn handle_cfg_matches(cfg_matches : &ArgMatches) {
    let grammar_file = cfg_matches.value_of("grammar").unwrap();
    let grammar_string = read_file(grammar_file.to_string());
    let g : CFG<String, String, LogDomain<f64>> = grammar_string.parse().unwrap();

    let a = PushDownAutomaton::from(g);

    let mut approx_matches = get_approx_args(cfg_matches);
    
    let ptk_string = "ptk".to_string();
    let rlb_string = "rlb".to_string();

    approx_matches.reverse();

    // TODO: this is not very elegant and it is only possible to parse 
    // explicitly programmed combinations of approximation strategies
    // the problem is that the strategies do not have the same type and
    // it is impossible to create a list of Box<dyn ApproximationStrategy<...>>
    // since ApproximationStrategy itself has generic parameters
    // currently possible:
    
    //      - rlb
    //      - rlb, ptk
    //      - ptk
    //      - ptk, rlb

    // match the first strategy
    match approx_matches.pop() {
        // no strategy
        None => {
            panic!("You need to choose at least one approximation strategy.");
        },

        Some((first_strategy, fst_additional)) => {
            if first_strategy == rlb_string {
                
                let rlb_file = fst_additional.unwrap();
                // create the rlb strategy
                let classes_string = read_file(rlb_file);

                let e: EquivalenceRelation<String, String> = classes_string.parse().unwrap();
                let f = |ps: &PushState<_,_>| ps.map(|nt| e.project(nt));
                let s1 = RlbElement::new(&f);

                // match the second strategy having rlb
                match approx_matches.pop() {
                    // no second strategy
                    // use original macro to create the CoarseToFineRecogniser
                    None => {
                        let recogniser = coarse_to_fine_recogniser!(a; s1);
                        recognise!(recogniser);
                    },

                    Some((second_strategy, sec_additional)) => {
                        if second_strategy == ptk_string {
                            
                            let k : usize = sec_additional.unwrap().parse().unwrap();

                            let s2 = PDTopKElement::new(k);

                            // try matching a third strategy which is currently not allowed
                            match approx_matches.pop() {
                                None => {
                                    let recogniser = coarse_to_fine_recogniser!(a; s1, s2);
                                    recognise!(recogniser);
                                },
                                Some(_) => {
                                    panic!("currently you are not allowed to use more than two strategies!");
                                },
                            }
                        } else {
                            panic!("{} not allowed here", second_strategy);
                        }
                    },
                }
            } else if first_strategy == ptk_string {
                
                // parse the first argument into an integer that limits the pushdown
                let k : usize = fst_additional.unwrap().parse().unwrap();

                // create the ptk strategy
                let s1 = PDTopKElement::new(k);

                // match the second strategy having ptk
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
                            // create rlb strategy
                            let classes_string = read_file(rlb_file);
                            
                            let e: EquivalenceRelation<String, String> = classes_string.parse().unwrap();
                            let f = |ps: &PushState<_,_>| ps.map(|nt| e.project(nt));
                            let s2 = RlbElement::new(&f);

                            // try matching a fourth strategy which is currently not allowed
                            match approx_matches.pop() {
                                None => {
                                    let recogniser = coarse_to_fine_recogniser!(a; s1, s2);
                                    recognise!(recogniser);
                                },
                                Some(_) => {
                                    panic!("currently you are not allowed to use more than two strategies!");
                                },
                            }
                        } else {
                            panic!("{} not allowed here or not implemented yet", second_strategy);
                        }
                    },
                    
                }
            } else {
                panic!("{} not allowed here", first_strategy);
            }
        }
    }
}

pub fn bench_mcfg(mcfg_matches : &ArgMatches) {
    let file_name = mcfg_matches.value_of("file_name").unwrap();
    let grammar_file = format!("{}{}", &file_name, ".gr");
    let equiv_file1 = format!("{}{}", &file_name, ".classes");
    let equiv_file2 = format!("{}{}", &file_name, "_2.classes");
    let corpus_file = format!("{}{}", &file_name, ".txt");

    let grammar_string = read_file(grammar_file);
    let equiv_string1 = read_file(equiv_file1);
    
    let corpus_string = read_file(corpus_file);

    let file_opt = File::open(equiv_file2).ok();
    let equiv_string2_opt = file_opt.map(|mut file| {
        let mut string = String::new();
        let _ = file.read_to_string(&mut string);
        string
    });

    let grammar : PMCFG<String, String, LogDomain<f64>> = grammar_string.parse().unwrap();
    let equiv_rel1 : EquivalenceRelation<String, String> = equiv_string1.parse().unwrap();
    let equiv_rel2_opt : Option<EquivalenceRelation<String, String>> = equiv_string2_opt.map(|equiv_string2| equiv_string2.parse().unwrap());
    
    
    let mut corpus : Vec<Vec<String>> = 
        corpus_string.lines()
                        .map(|l| l.split_whitespace()
                                    .map(|w| w.to_string())
                                    .collect())
                        .filter(|l : &Vec<String>| !l.is_empty())
                        .collect();
    // corpus.sort();
    corpus.sort_by(|a, b| a.len().cmp(&b.len()));
    // let sentences = corpus.iter().take(10);
    let corpus = corpus.iter().take(5);

    println!("tested sentences: {:?}", &corpus);

    for sentence in corpus.clone() {
        println!("{:?}", &sentence);
    
        benchmarks::bench_mcfg(
            file_name.to_string(),
            grammar.clone(),
            sentence.clone()
        );

        benchmarks::bench_mcfg_tts(
            file_name.to_string(),
            grammar.clone(),
            sentence.clone()
        );

        benchmarks::bench_mcfg_tts_rlb(
            file_name.to_string(),
            grammar.clone(),
            equiv_rel1.clone(),
            sentence.clone()
        );

        match equiv_rel2_opt.clone() {
            Some(equiv_rel2) => {
                let my_file_name = format!("{}{}", &file_name, "_2");
                benchmarks::bench_mcfg_tts_rlb(
                    my_file_name.to_string(),
                    grammar.clone(),
                    equiv_rel2,
                    sentence.clone()
                );
            },
            None => (),
        }

        benchmarks::bench_mcfg_rlb(
            file_name.to_string(),
            grammar.clone(),
            equiv_rel1.clone(),
            sentence.clone()
        );

        match equiv_rel2_opt.clone() {
            Some(equiv_rel2) => {
                let my_file_name = format!("{}{}", &file_name, "_2");
                benchmarks::bench_mcfg_rlb(
                    my_file_name.to_string(),
                    grammar.clone(),
                    equiv_rel2,
                    sentence.clone()
                );
            },
            None => (),
        }

        benchmarks::bench_mcfg_rlb_tts(
            file_name.to_string(),
            grammar.clone(),
            equiv_rel1.clone(),
            sentence.clone()
        );

        match equiv_rel2_opt.clone() {
            Some(equiv_rel2) => {
                let my_file_name = format!("{}{}", &file_name, "_2");
                benchmarks::bench_mcfg_rlb_tts(
                    my_file_name.to_string(),
                    grammar.clone(),
                    equiv_rel2,
                    sentence.clone()
                );
            },
            None => (),
        }
    }


    for ptk in vec![20,15] {
        for sentence in corpus.clone() {
            println!("{:?}", &sentence);

            benchmarks::bench_mcfg_tts_rlb_ptk(
                file_name.to_string(),
                grammar.clone(),
                equiv_rel1.clone(),
                sentence.clone(),
                ptk
            );

            match equiv_rel2_opt.clone() {
                Some(equiv_rel2) => {
                    let my_file_name = format!("{}{}", &file_name, "_2");
                    benchmarks::bench_mcfg_tts_rlb_ptk(
                        my_file_name.to_string(),
                        grammar.clone(),
                        equiv_rel2,
                        sentence.clone(),
                        ptk
                    );
                },
                None => (),
            }

            benchmarks::bench_mcfg_tts_ptk(
                file_name.to_string(),
                grammar.clone(),
                sentence.clone(),
                ptk
            );

            benchmarks::bench_mcfg_tts_ptk_rlb(
                file_name.to_string(),
                grammar.clone(),
                equiv_rel1.clone(),
                sentence.clone(),
                ptk
            );

            match equiv_rel2_opt.clone() {
                Some(equiv_rel2) => {
                    let my_file_name = format!("{}{}", &file_name, "_2");
                    benchmarks::bench_mcfg_tts_ptk_rlb(
                        my_file_name.to_string(),
                        grammar.clone(),
                        equiv_rel2,
                        sentence.clone(),
                        ptk
                    );
                },
                None => (),
            }

            benchmarks::bench_mcfg_rlb_tts_ptk(
                file_name.to_string(),
                grammar.clone(),
                equiv_rel1.clone(),
                sentence.clone(),
                ptk
            );

            match equiv_rel2_opt.clone() {
                Some(equiv_rel2) => {
                    let my_file_name = format!("{}{}", &file_name, "_2");
                    benchmarks::bench_mcfg_rlb_tts_ptk(
                        my_file_name.to_string(),
                        grammar.clone(),
                        equiv_rel2,
                        sentence.clone(),
                        ptk
                    );
                },
                None => (),
            }
        }
    }
}

#[test]
fn test_sortby() {
    let mut v = vec![vec![4,4,4,4],vec![2,2], vec![4], vec![1,1,1]];
    v.sort_by(|a, b| a.len().cmp(&b.len()));
    assert_eq!(v, vec![vec![4], vec![2,2], vec![1,1,1], vec![4,4,4,4]]);
}

// handle a given mcfg grammar with a tsa
pub fn handle_mcfg_matches(mcfg_matches : &ArgMatches) {


    let grammar_file = mcfg_matches.value_of("grammar").unwrap();
    let grammar_string = read_file(grammar_file.to_string());
    let g : PMCFG<String, String, LogDomain<f64>> = grammar_string.parse().unwrap();

    let a = TreeStackAutomaton::from(g);

    let mut approx_matches = get_approx_args(mcfg_matches);
    
    let tts_string = "tts".to_string();
    let ptk_string = "ptk".to_string();
    let rlb_string = "rlb".to_string();

    approx_matches.reverse();

    // TODO: this is not very elegant and it is only possible to parse 
    // explicitly programmed combinations of approximation strategies
    // the problem is that the strategies do not have the same type and
    // it is impossible to create a list of Box<dyn ApproximationStrategy<...>>
    // since ApproximationStrategy itself has generic parameters
    // currently possible:
    //      - tts
    //      - tts, rlb
    //      - tts, rlb, ptk
    //      - tts, ptk
    //      - tts, ptk, rlb
    //      - rlb
    //      - rlb, tts
    //      - rlb, tts, ptk

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
                            //let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = classes_string.parse().unwrap();
                            let e: EquivalenceRelation<String, String> = classes_string.parse().unwrap();
                            let f = |ps: &PosState<PMCFGRule<_,_,_>>| ps.map(|r| r.map_nonterminals(|nt| e.project(nt)));
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
                                        
                                        let k : usize = trd_additional.unwrap().parse().unwrap();

                                        let s3 = PDTopKElement::new(k);

                                        // try matching a fourth strategy which is currently not allowed
                                        match approx_matches.pop() {
                                            None => {
                                                let recogniser = coarse_to_fine_recogniser!(a; s1, s2, s3);
                                                recognise!(recogniser);
                                            },
                                            Some(_) => {
                                                panic!("currently you are not allowed to use more than three strategies!");
                                            },
                                        }
                                    } else {
                                        panic!("{} not allowed here", third_strategy);
                                    }
                                },
                            }
                        } else if second_strategy == ptk_string {
                            
                            // parse the second argument into an integer that limits the pushdown
                            let k : usize = sec_additional.unwrap().parse().unwrap();

                            // create the ptk strategy
                            let s2 = PDTopKElement::new(k);

                            // match the third strategy having tts, ptk
                            match approx_matches.pop() {
                                // no third strategy
                                // use original macro to create the CoarseToFineRecogniser
                                None => {
                                    let recogniser = coarse_to_fine_recogniser!(a; s1, s2);
                                    recognise!(recogniser);
                                },
                                Some((third_strategy, trd_additional)) => {
                                    if third_strategy == rlb_string {
                                        
                                        let rlb_file = trd_additional.unwrap();
                                        // create rlb strategy
                                        let classes_string = read_file(rlb_file);
                                        let e: EquivalenceRelation<String, String> = classes_string.parse().unwrap();
                                        let f = |ps: &PosState<PMCFGRule<_,_,_>>| ps.map(|r| r.map_nonterminals(|nt| e.project(nt)));
                                        let s3 = RlbElement::new(&f);

                                        // try matching a fourth strategy which is currently not allowed
                                        match approx_matches.pop() {
                                            None => {
                                                let recogniser = coarse_to_fine_recogniser!(a; s1, s2, s3);
                                                recognise!(recogniser);
                                            },
                                            Some(_) => {
                                                panic!("currently you are not allowed to use more than three strategies!");
                                            },
                                        }
                                    } else {
                                        panic!("{} not allowed here or not implemented yet", third_strategy);
                                    }
                                },
                                
                            }
                        } else {
                            panic!("{} not allowed here", second_strategy);
                        }
                    }
                }
            } else if first_strategy == rlb_string {
                
                let rlb_file = fst_additional.unwrap();

                // create the rlb strategy
                let classes_string = read_file(rlb_file);
                let e: EquivalenceRelation<String, String> = classes_string.parse().unwrap();
                let f = |ps: &PosState<PMCFGRule<_,_,_>>| ps.map(|r| r.map_nonterminals(|nt| e.project(nt)));
                let s1 = RlbElementTSA::new(&f);

                // match the second strategy having rlb as first strategy
                match approx_matches.pop() {
                    // no second strategy 
                    // use original macro to create CoarseToFineRecogniser
                    None => {
                        let recogniser = coarse_to_fine_recogniser!(a; s1);
                        recognise!(recogniser);
                        //recogniser.recognise(vec!["a","a","c","c"].iter().map(|a| a.to_string()).collect());
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
                                        

                                        let k : usize = trd_additional.unwrap().parse().unwrap();

                                        let s3 = PDTopKElement::new(k);

                                        // try matching a fourth strategy which is currently not allowed
                                        match approx_matches.pop() {
                                            None => {
                                                let recogniser = coarse_to_fine_recogniser!(a; s1, s2, s3);
                                                recognise!(recogniser);
                                            },
                                            Some(_) => {
                                                panic!("currently you are not allowed to use more than three strategies!");
                                            },
                                        }
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
        _ => panic!("You need to choose at least one approximation strategy."),
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
    let _ = file.read_to_string(&mut string);
    string
}
