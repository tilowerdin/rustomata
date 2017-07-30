extern crate clap;
#[macro_use]
extern crate nom;
extern crate num_traits;

mod automata;
mod tree_stack;
mod pmcfg;
mod util;

mod push_down;
mod cfg;
mod approximation;
mod integerise;
mod nfa;

#[cfg(test)]
mod tests;

use clap::{Arg, App, SubCommand};
use std::io::prelude::*;
use std::fs::File;

pub use tree_stack::*;
pub use automata::*;
pub use pmcfg::*;
pub use util::*;
use util::equivalence_classes::*;
pub use util::ctf::*;

pub use push_down::*;
pub use cfg::*;
pub use approximation::*;
pub use integerise::*;

fn main() {
    let matches
        = App::new("rustomata")
        .version("0.1")
        .author("Tobias Denkinger <tobias.denkinger@tu-dresden.de>")
        .about("Framework for (weighted) automata with storage")
        .subcommand(SubCommand::with_name("mcfg")
                    .about("functions related to multiple context-free grammars")
                    .subcommand(SubCommand::with_name("parse")
                                .about("parses from stdin with a multiple context-free grammar")
                                .arg(Arg::with_name("grammar")
                                     .help("grammar file to use")
                                     .index(1)
                                     .required(true))
                                .arg(Arg::with_name("number-of-parses")
                                     .help("number of parses that should be returned")
                                     .short("n")
                                     .long("number")
                                     .default_value("1")
                                     .required(false)))
                    .subcommand(SubCommand::with_name("automaton")
                                .about("constructs a tree-stack automaton from the given multiple context-free grammar")
                                .arg(Arg::with_name("grammar")
                                     .help("grammar file to use")
                                     .index(1)
                                     .required(true))))
        .subcommand(SubCommand::with_name("cfg")
                    .about("functions related to context-free grammars")
                    .subcommand(SubCommand::with_name("parse")
                                .about("parses a word given a multiple context-free grammar")
                                .arg(Arg::with_name("grammar")
                                     .help("grammar file to use")
                                     .index(1)
                                     .required(true))
                                .arg(Arg::with_name("number-of-parses")
                                     .help("number of parses that should be returned")
                                     .short("n")
                                     .long("number")
                                     .default_value("1")
                                     .required(false)))
                    .subcommand(SubCommand::with_name("automaton")
                                .about("constructs a tree-stack automaton from the given multiple context-free grammar")
                                .arg(Arg::with_name("grammar")
                                        .help("grammar file to use")
                                        .index(1)
                                        .required(true))))
        .subcommand(SubCommand::with_name("tsa")
                    .about("functions related to tree-stack automata")
                    .subcommand(SubCommand::with_name("recognise")
                                .about("recognises from stdin with a tree-stack automaton")
                                .arg(Arg::with_name("automaton")
                                     .help("automaton file to use")
                                     .index(1)
                                     .required(true))
                                .arg(Arg::with_name("number-of-runs")
                                     .help("number of runs that should be returned")
                                     .short("n")
                                     .long("number")
                                     .default_value("1")
                                     .required(false))))
        .subcommand(SubCommand::with_name("coarse-to-fine")
                    .about("functions related to the coarse-to-fine recognise approach")
                    .subcommand(SubCommand::with_name("cfg")
                                .about("coarse-to-fine recognising using push-down automata")
                                .subcommand(SubCommand::with_name("parse")
                                            .about("parses a word given a context-free grammar")
                                            .arg(Arg::with_name("grammar")
                                                    .help("grammar file to use")
                                                    .index(1)
                                                    .required(true))
                                            .arg(Arg::with_name("classes")
                                                    .help("classes file to use")
                                                    .index(2)
                                                    .required(true))
                                            .arg(Arg::with_name("topk-size")
                                                    .help("size of the limited push-down")
                                                    .index(3)
                                                    .required(true))
                                            .arg(Arg::with_name("number-of-parses")
                                                    .help("number of parses that should be returned")
                                                    .short("n")
                                                    .long("number")
                                                    .default_value("1")
                                                    .required(false)))
                                .subcommand(SubCommand::with_name("automaton")
                                            .about("constructs a number of push-down automata from the given context-free grammar")
                                            .arg(Arg::with_name("grammar")
                                                    .help("grammar file to use")
                                                    .index(1)
                                                    .required(true))
                                            .arg(Arg::with_name("classes")
                                                    .help("classes file to use")
                                                    .index(2)
                                                    .required(true))
                                            .arg(Arg::with_name("topk-size")
                                                    .help("size of the limited push-down")
                                                    .index(3)
                                                    .required(true))))
                    .subcommand(SubCommand::with_name("mcfg")
                                .about("coarse-to-fine recognising using push-down and tree-stack automata")
                                .subcommand(SubCommand::with_name("parse")
                                            .about("parses a word given a context-free grammar")
                                            .arg(Arg::with_name("grammar")
                                                    .help("grammar file to use")
                                                    .index(1)
                                                    .required(true))
                                            .arg(Arg::with_name("classes")
                                                    .help("classes file to use")
                                                    .index(2)
                                                    .required(true))
                                            .arg(Arg::with_name("topk-size")
                                                    .help("size of the limited push-down")
                                                    .index(3)
                                                    .required(true))
                                            .arg(Arg::with_name("number-of-parses")
                                                    .help("number of parses that should be returned")
                                                    .short("n")
                                                    .long("number")
                                                    .default_value("1")
                                                    .required(false)))
                                .subcommand(SubCommand::with_name("automaton")
                                            .about("constructs a number of push-down automata from the given context-free grammar")
                                            .arg(Arg::with_name("grammar")
                                                    .help("grammar file to use")
                                                    .index(1)
                                                    .required(true))
                                            .arg(Arg::with_name("classes")
                                                    .help("classes file to use")
                                                    .index(2)
                                                    .required(true))
                                            .arg(Arg::with_name("topk-size")
                                                    .help("size of the limited push-down")
                                                    .index(3)
                                                    .required(true)))))
        .subcommand(SubCommand::with_name("approximation")
                    .about("functions related to single approximations")
                    .subcommand(SubCommand::with_name("relabel")
                                .about("relabels automata with classes file")
                                .subcommand(SubCommand::with_name("parse")
                                        .arg(Arg::with_name("classes")
                                                .help("classes file to use")
                                                .index(1)
                                                .required(true))
                                        .arg(Arg::with_name("grammar")
                                                .help("cfg-grammar file to use")
                                                .index(2)
                                                .required(true)))
                                .subcommand(SubCommand::with_name("automaton")
                                        .arg(Arg::with_name("classes")
                                                .help("classes file to use")
                                                .index(1)
                                                .required(true))
                                        .arg(Arg::with_name("grammar")
                                                .help("cfg-grammar file to use")
                                                .index(2)
                                                .required(true))))
                    .subcommand(SubCommand::with_name("topk")
                                .about("maps pushdown to its topmost k elements")
                                .subcommand(SubCommand::with_name("parse")
                                        .arg(Arg::with_name("grammar")
                                                .help("cfg-grammar file to use")
                                                .index(2)
                                                .required(true))
                                        .arg(Arg::with_name("size")
                                                .help("size of pushdown")
                                                .index(1)
                                                .required(true)))
                                .subcommand(SubCommand::with_name("automaton")
                                        .arg(Arg::with_name("grammar")
                                                .help("cfg-grammar file to use")
                                                .index(2)
                                                .required(true))
                                        .arg(Arg::with_name("size")
                                                .help("size of pushdown")
                                                .index(1)
                                                .required(true))))
                    .subcommand(SubCommand::with_name("tts")
                                .about("approximates tree-stack into pushdown automata")
                                .subcommand(SubCommand::with_name("parse")
                                        .arg(Arg::with_name("grammar")
                                                .help("mcfg-grammar file to use")
                                                .index(1)
                                                .required(true)))
                                .subcommand(SubCommand::with_name("automaton")
                                        .arg(Arg::with_name("grammar")
                                                .help("mcfg-grammar file to use")
                                                .index(1)
                                                .required(true)))))
        .get_matches();

    match matches.subcommand() {
        ("mcfg", Some(mcfg_matches)) => {
            match mcfg_matches.subcommand() {
                ("parse", Some(mcfg_parse_matches)) => {
                    let grammar_file_name = mcfg_parse_matches.value_of("grammar").unwrap();
                    let mut grammar_file = File::open(grammar_file_name).unwrap();
                    let n = mcfg_parse_matches.value_of("number-of-parses").unwrap().parse().unwrap();
                    let mut grammar_string = String::new();
                    let _ = grammar_file.read_to_string(&mut grammar_string);
                    let grammar: PMCFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                    let automaton = IntTreeStackAutomaton::from(grammar);

                    let mut corpus = String::new();
                    let _ = std::io::stdin().read_to_string(&mut corpus);

                    for sentence in corpus.lines() {
                        println!("{}:", sentence);
                        for parse in automaton.recognise(sentence.split_whitespace().map(|x| x.to_string()).collect()).take(n) {
                            println!("{:?}", parse.translate().0);
                        }
                        println!();
                    }
                },
                ("automaton", Some(mcfg_automaton_matches)) => {
                    let grammar_file_name = mcfg_automaton_matches.value_of("grammar").unwrap();
                    let mut grammar_file = File::open(grammar_file_name).unwrap();
                    let mut grammar_string = String::new();
                    let _ = grammar_file.read_to_string(&mut grammar_string);
                    let grammar: PMCFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                    let automaton = IntTreeStackAutomaton::from(grammar);
                    println!("{}", automaton);
                }
                _ => ()
            }
        },
        ("cfg", Some(cfg_matches)) => {
            match cfg_matches.subcommand() {
                ("parse", Some(cfg_parse_matches)) => {
                    let grammar_file_name = cfg_parse_matches.value_of("grammar").unwrap();
                    let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                    let n = cfg_parse_matches.value_of("number-of-parses").unwrap().parse().unwrap();
                    let mut grammar_string = String::new();
                    let _ = grammar_file.read_to_string(&mut grammar_string);
                    let grammar: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                    let automaton = IntPushDownAutomaton::from(grammar);

                    let mut corpus = String::new();
                    let _ = std::io::stdin().read_to_string(&mut corpus);

                    for sentence in corpus.lines() {
                        println!("{}:", sentence);
                        for parse in automaton.recognise(sentence.split_whitespace().map(|x| x.to_string()).collect()).take(n) {
                            println!("{:?}", parse.translate().0);
                        }
                        println!();
                    }
                },
                ("automaton", Some(cfg_automaton_matches)) => {
                    let grammar_file_name = cfg_automaton_matches.value_of("grammar").unwrap();
                    let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                    let mut grammar_string = String::new();
                    let _ = grammar_file.read_to_string(&mut grammar_string);
                    let grammar: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                    let automaton = IntPushDownAutomaton::from(grammar);
                    println!("{}", automaton);
                },
                _ => ()
            }
        },
        ("coarse-to-fine", Some(ctf_matches)) => {
            match ctf_matches.subcommand() {
                ("cfg", Some(cfg_matches)) => {
                    match cfg_matches.subcommand() {
                        ("parse", Some(cfg_parse_matches)) => {
                            let grammar_file_name = cfg_parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let n = cfg_parse_matches.value_of("number-of-parses").unwrap().parse().unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let grammar: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let a = IntPushDownAutomaton::from(grammar);

                            let classes_file_name = cfg_parse_matches.value_of("classes").unwrap();
                            let mut classes_file = File::open(classes_file_name.clone()).unwrap();
                            let mut classes_string = String::new();
                            let _ = classes_file.read_to_string(&mut classes_string);
                            let e: EquivalenceClass<String, String> = classes_string.parse().unwrap();

                            let rlb = RlbElement::new(e);

                            let (b, nrlb) = a.approximation(&rlb).unwrap();

                            let size = cfg_parse_matches.value_of("topk-size").unwrap().parse::<usize>().unwrap();

                            let ptk = PDTopKElement::new(size);

                            let (c, nptk) = b.approximation(&ptk).unwrap();

                            let mut corpus = String::new();
                            let _ = std::io::stdin().read_to_string(&mut corpus);

                            let n1 = 100;
                            let n2 = 10;
                            let n3 = n;
                            let mut c2 = 0;
                            let mut c3 = 0;

                            for sentence in corpus.lines() {
                                println!("{}:\n", sentence);
                                let sentence2 = sentence.clone();
                                let word = sentence.split_whitespace().map(|x| x.to_string()).collect();
                                for parse1 in c.recognise(sentence2.split_whitespace().map(|x| x.to_string()).collect()).take(n1) {
                                    let s1 = ctf_level_i(&word, parse1.give_up().1, &nptk, &b);
                                    for parse2 in s1{
                                        let s2 = ctf_level_i(&word, parse2.give_up().1, &nrlb, &a);
                                        for parse3 in s2{
                                            println!("{}", Run::new(parse3.translate().1));
                                            c3=c3+1;
                                            if c3>=n3{
                                                break
                                            }
                                        }
                                        c2=c2+1;
                                        if c2>=n2||c3>=n3{
                                            break;
                                        }
                                    }
                                    if c2>=n2||c3>=n3{
                                        break;
                                    }
                                }
                            }
                        },
                        ("automaton", Some(cfg_automaton_matches)) => {
                            let grammar_file_name = cfg_automaton_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let grammar: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let a = IntPushDownAutomaton::from(grammar);
                            println!("Original Automaton: \n\n{}", a);

                            let classes_file_name = cfg_automaton_matches.value_of("classes").unwrap();
                            let mut classes_file = File::open(classes_file_name.clone()).unwrap();
                            let mut classes_string = String::new();
                            let _ = classes_file.read_to_string(&mut classes_string);
                            let e: EquivalenceClass<String, String> = classes_string.parse().unwrap();

                            let rlb = RlbElement::new(e);

                            let (b, _) = a.approximation(&rlb).unwrap();

                            println!("Step 1 (relabel): \n\n{}", b);

                            let size = cfg_automaton_matches.value_of("topk-size").unwrap().parse::<usize>().unwrap();

                            let ptk = PDTopKElement::new(size);

                            let (c, _) = b.approximation(&ptk).unwrap();

                            println!("Step 2 (restrict to size): \n\n{}", c);
                        },
                        _ => ()
                    }
                },
                ("mcfg", Some(mcfg_matches)) => {
                    match mcfg_matches.subcommand() {
                        ("parse", Some(mcfg_parse_matches)) => {
                            let grammar_file_name = mcfg_parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name).unwrap();
                            let n = mcfg_parse_matches.value_of("number-of-parses").unwrap().parse().unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let grammar: PMCFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let automaton = IntTreeStackAutomaton::from(grammar);

                            let tts = TTSElement::new();

                            let (a, ntts) = automaton.approximation(&tts).unwrap();

                            let classes_file_name = mcfg_parse_matches.value_of("classes").unwrap();
                            let mut classes_file = File::open(classes_file_name.clone()).unwrap();
                            let mut classes_string = String::new();
                            let _ = classes_file.read_to_string(&mut classes_string);
                            let e: EquivalenceClass<String, String> = classes_string.parse().unwrap();

                            let rlb = RlbElement::new(e);

                            let (b, nrlb) = a.approximation(&rlb).unwrap();

                            let size = mcfg_parse_matches.value_of("topk-size").unwrap().parse::<usize>().unwrap();

                            let ptk = PDTopKElement::new(size);

                            let (c, nptk) = b.approximation(&ptk).unwrap();

                            let mut corpus = String::new();
                            let _ = std::io::stdin().read_to_string(&mut corpus);

                            let n1 = 1000;
                            let n2 = 100;
                            let n3 = 10;
                            let n4 = n;
                            let mut c2 = 0;
                            let mut c3 = 0;
                            let mut c4 = 0;

                            for sentence in corpus.lines() {
                                println!("{}:\n", sentence);
                                let sentence2 = sentence.clone();
                                let word = sentence.split_whitespace().map(|x| x.to_string()).collect();
                                for parse1 in c.recognise(sentence2.split_whitespace().map(|x| x.to_string()).collect()).take(n1) {
                                    let s1 = ctf_level_i(&word, parse1.give_up().1, &nptk, &b);
                                    for parse2 in s1{
                                        let s2 = ctf_level_i(&word, parse2.give_up().1, &nrlb, &a);
                                        for parse3 in s2{
                                            let s3 = ctf_level_i(&word, parse3.give_up().1, &ntts, &automaton);
                                            for parse4 in s3{
                                                println!("{}", Run::new(parse4.translate().1));
                                                c4=c4+1;
                                                if c4>=n4{
                                                    break
                                                }
                                            }
                                            c3=c3+1;
                                            if c4>=n4||c3>=n3{
                                                break;
                                            }
                                        }
                                        c2=c2+1;
                                        if c2>=n2||c3>=n3||c4>=n4{
                                            break;
                                        }
                                    }
                                    if c2>=n2||c3>=n3||c4>=n4{
                                        break;
                                    }
                                }
                            }
                        },
                        ("automaton", Some(mcfg_automaton_matches)) => {
                            let grammar_file_name = mcfg_automaton_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let grammar: PMCFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let automaton = IntTreeStackAutomaton::from(grammar);
                            println!("Original Automaton: \n\n{}", automaton);

                            let tts = TTSElement::new();

                            let (a, _) = automaton.approximation(&tts).unwrap();

                            println!("Step 1 (transform to push-down): \n\n{}", a);

                            let classes_file_name = mcfg_automaton_matches.value_of("classes").unwrap();
                            let mut classes_file = File::open(classes_file_name.clone()).unwrap();
                            let mut classes_string = String::new();
                            let _ = classes_file.read_to_string(&mut classes_string);
                            let e: EquivalenceClass<String, String> = classes_string.parse().unwrap();

                            let rlb = RlbElement::new(e);

                            let (b, _) = a.approximation(&rlb).unwrap();

                            println!("Step 2 (relabel): \n\n{}", b);

                            let size = mcfg_automaton_matches.value_of("topk-size").unwrap().parse::<usize>().unwrap();

                            let ptk = PDTopKElement::new(size);


                            let (c, _)= b.approximation(&ptk).unwrap();

                            println!("Step 3 (restrict to size): \n\n{}", c);
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
        },
        ("tsa", Some(tsa_matches)) => {
            match tsa_matches.subcommand() {
                ("recognise", Some(tsa_recognise_matches)) => {
                    let automaton_file_name = tsa_recognise_matches.value_of("automaton").unwrap();
                    let mut automaton_file = File::open(automaton_file_name).unwrap();
                    let n = tsa_recognise_matches.value_of("number-of-runs").unwrap().parse().unwrap();
                    let mut automaton_string = String::new();
                    let _ = automaton_file.read_to_string(&mut automaton_string);
                    let automaton: TreeStackAutomaton<String, String, util::log_prob::LogProb> = automaton_string.parse().unwrap();

                    let mut corpus = String::new();
                    let _ = std::io::stdin().read_to_string(&mut corpus);

                    for sentence in corpus.lines() {
                        println!("{}:", sentence);
                        for run in automaton.recognise(sentence.split_whitespace().map(|x| x.to_string()).collect()).take(n) {
                            println!("{:?}", run.1);
                        }
                        println!();
                    }
                },
                _ => ()
            }
        },
        ("approximation", Some(r_matches)) => {
            match r_matches.subcommand() {
                ("relabel",Some(relabel_matches)) => {
                    match relabel_matches.subcommand(){
                        ("parse",Some(parse_matches)) => {
                            let grammar_file_name = parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let g: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let a = IntPushDownAutomaton::from(g);

                            let classes_file_name = parse_matches.value_of("classes").unwrap();
                            let mut classes_file = File::open(classes_file_name.clone()).unwrap();
                            let mut classes_string = String::new();
                            let _ = classes_file.read_to_string(&mut classes_string);
                            let e: EquivalenceClass<String, String> = classes_string.parse().unwrap();

                            let rlb = RlbElement::new(e);

                            let (b, _) = a.approximation(&rlb).unwrap();

                            let mut corpus = String::new();
                            let _ = std::io::stdin().read_to_string(&mut corpus);

                            for sentence in corpus.lines() {
                                println!("{:?}: {}",
                                         b.recognise(sentence.split_whitespace().map(|x| x.to_string()).collect()).next(),
                                         sentence);
                            }
                        },
                        ("automaton",Some(parse_matches)) => {
                            let grammar_file_name = parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let g: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let a = IntPushDownAutomaton::from(g);

                            println!("Original Automaton");
                            println!("{}", a);

                            let classes_file_name = parse_matches.value_of("classes").unwrap();
                            let mut classes_file = File::open(classes_file_name.clone()).unwrap();
                            let mut classes_string = String::new();
                            let _ = classes_file.read_to_string(&mut classes_string);
                            let e: EquivalenceClass<String, String> = classes_string.parse().unwrap();

                            let rlb = RlbElement::new(e);

                            let (b, _) = a.approximation(&rlb).unwrap();

                            println!("Approximated Automaton");
                            println!("{}", b);
                        },
                        _ => ()
                    }

                },
                ("topk",Some(topk_matches)) => {
                    match topk_matches.subcommand(){
                        ("parse",Some(parse_matches)) => {
                            let grammar_file_name = parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let g: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let size = parse_matches.value_of("size").unwrap().parse::<usize>().unwrap();

                            let a = IntPushDownAutomaton::from(g);

                            let ptk = PDTopKElement::new(size);

                            let (b, _) = a.approximation(&ptk).unwrap();

                            let mut corpus = String::new();
                            let _ = std::io::stdin().read_to_string(&mut corpus);

                            for sentence in corpus.lines() {
                                println!("{:?}: {}",
                                         b.recognise(sentence.split_whitespace().map(|x| x.to_string()).collect()).next(),
                                         sentence);
                            }
                        },
                        ("automaton",Some(parse_matches)) => {
                            let grammar_file_name = parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let g: CFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let size = parse_matches.value_of("size").unwrap().parse::<usize>().unwrap();

                            let a = IntPushDownAutomaton::from(g);

                            println!("Original Automaton");
                            println!("{}", a);

                            let ptk = PDTopKElement::new(size);

                            let (b, _) = a.approximation(&ptk).unwrap();
                            println!("Approximated Automaton");
                            println!("{}", b);
                        },
                        _ => ()
                    }

                },
                ("tts",Some(tts_matches)) => {
                    match tts_matches.subcommand(){
                        ("parse",Some(parse_matches)) => {
                            let grammar_file_name = parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let g: PMCFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let a = IntTreeStackAutomaton::from(g);

                            let tts = TTSElement::new();

                            let (b, _) = a.approximation(&tts).unwrap();

                            let mut corpus = String::new();
                            let _ = std::io::stdin().read_to_string(&mut corpus);

                            for sentence in corpus.lines() {
                                println!("{:?}", sentence);
                                println!("{:?}: {}",
                                        b.recognise(sentence.split_whitespace().map(|x| x.to_string()).collect()).next(),
                                        sentence);
                            }
                        },
                        ("automaton",Some(parse_matches)) => {
                            let grammar_file_name = parse_matches.value_of("grammar").unwrap();
                            let mut grammar_file = File::open(grammar_file_name.clone()).unwrap();
                            let mut grammar_string = String::new();
                            let _ = grammar_file.read_to_string(&mut grammar_string);
                            let g: PMCFG<String, String, util::log_prob::LogProb> = grammar_string.parse().unwrap();

                            let a = IntTreeStackAutomaton::from(g);
                            println!("Original Automaton");
                            println!("{}", a);

                            let tts = TTSElement::new();

                            let (b, _) = a.approximation(&tts).unwrap();
                            println!("Approximated Automaton");
                            println!("{}", b);
                        },
                        _ => ()
                    }

                },
                _ => ()
            }
        },
        _ => ()
    }

}
