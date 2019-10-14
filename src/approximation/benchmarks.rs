
use crate::automata::tree_stack_automaton::TreeStackAutomaton;
use crate::recognisable::automaton::Automaton;
use crate::recognisable::Item;
use crate::automata::tree_stack_automaton::PosState;
use crate::grammars::pmcfg::PMCFGRule;
use crate::approximation::equivalence_classes::EquivalenceRelation;
use log_domain::LogDomain;
use crate::grammars::pmcfg::PMCFG;
use crate::approximation::relabel::{RlbElement,RlbElementTSA};
use crate::approximation::tts::TTSElement;
use crate::approximation::ptk::PDTopKElement;
use crate::approximation::ApproximationStrategy;
use crate::recognisable::Recognisable;
use crate::coarse_to_fine_recogniser;
use std::rc::Rc;
use crate::recognisable::coarse_to_fine::CoarseToFineRecogniser;

extern crate test;

use num_traits::pow::pow;

//extern crate time;

use std::fs::{self, File, OpenOptions};
use std::io::SeekFrom;
use std::io::prelude::*;
use std::time::{Duration,SystemTime};

const GRAMMAR_STRING : &str = "
initial: [S]

S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)
A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5
A → [[],  []                             ] (    )   # 0.5
B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5
B → [[],  []                             ] (    )   # 0.5
";

const CLASSES_STRING1 : &str = "
S [\"S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\"]
A [\"A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\", \"A → [[],  []] (    )   # 0.5\"]
B [\"B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\", \"B → [[],  []] (    )   # 0.5\"]
R *
";

const CLASSES_STRING2 : &str = "
R *
";

fn get_word1() -> Vec<String> {
    vec!["a","a","c","c"].into_iter().map(|x| x.to_string()).collect()
}

#[bench]
fn bench_no_approximation(b: &mut test::Bencher) {
    b.iter(|| {
        let g : PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

        let a = TreeStackAutomaton::from(g); 

        let word = get_word1();

        a.recognise(word);
    })
}

#[bench]
fn bench_tts(b: &mut test::Bencher) {
    b.iter(|| {
        let g : PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

        let a = TreeStackAutomaton::from(g); 

        let s1 = TTSElement::new();

        let recogniser = coarse_to_fine_recogniser!(a; s1);

        let word = get_word1();

        recogniser.recognise(word);
    })
}

#[bench]
fn bench_rlb1(b: &mut test::Bencher) {
    b.iter(|| {
        let g : PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

        let a = TreeStackAutomaton::from(g); 

        let e : EquivalenceRelation<PMCFGRule<_,_,_>, String> = CLASSES_STRING1.parse().unwrap();
        let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));

        let s1 = RlbElementTSA::new(&f);

        let recogniser = coarse_to_fine_recogniser!(a; s1);

        let word = get_word1();

        recogniser.recognise(word);
    })
}

#[bench]
fn bench_rlb2(b: &mut test::Bencher) {
    b.iter(|| {
        let g : PMCFG<String, String, LogDomain<f64>> = GRAMMAR_STRING.parse().unwrap();

        let a = TreeStackAutomaton::from(g); 

        let e : EquivalenceRelation<PMCFGRule<_,_,_>, String> = CLASSES_STRING2.parse().unwrap();
        let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));

        let s1 = RlbElementTSA::new(&f);

        let recogniser = coarse_to_fine_recogniser!(a; s1);

        let word = get_word1();

        recogniser.recognise(word);
    })
}

fn write_to_file(file_name : String, data : String) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)
        .unwrap();

    file.seek(SeekFrom::End(0));
    writeln!(file, "{}", data);
}

#[test]
fn test_write_to_file() {
    let file_name = "test_write_to_file.txt".to_string();
    let data1 = "if this text ".to_string(); 
    let data2 = "is in test_write_to_file.txt, the test was successful".to_string();
    write_to_file(file_name.clone(), data1);
    write_to_file(file_name, data2);
}

macro_rules! bench {
    ($rec:expr, $file:expr, $sentence:expr) => {
        
        write_to_file($file, "".to_string());
        let mut times : Vec<u128> = Vec::new();

        for _ in 0..5 {
            let my_sentence = $sentence.clone();
            let start = SystemTime::now();

            let parses = $rec.recognise(my_sentence);

            for _ in parses {
                break;
            }

            let end = SystemTime::now();

            let diff = end.duration_since(start).unwrap();

            times.push(diff.as_micros());
        }

        let (middle, sigma) = mid_sig(times);

        write_to_file($file, format!("middle: {}", middle).to_string());
        write_to_file($file, format!("sigma: {}", sigma).to_string());
        
    }
}


fn mid_sig(vec : Vec<u128>) -> (f64, f64) {
    let mut sum : u128 = vec.iter().sum();
    let mut middle = sum as f64 / vec.len() as f64;
    let mut sum: f64= vec.iter().map(|t| (*t as f64 - middle).powf(2.0)).sum();
    let mut var : f64 = sum / vec.len() as f64;
    let mut sigma : f64 = var.sqrt();

    (middle, sigma)
}

pub fn bench_mcfg(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    sentences : Vec<String>
    ) {
    println!("bench_mcfg");
    let file_name_string = format!("{}{}", &file_name, "_bench_mcfg.txt");
    
    let a = TreeStackAutomaton::from(grammar);

    bench!(a, file_name_string.clone(), sentences);
}

pub fn bench_mcfg_tts(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    sentences : Vec<String>
    ){
    println!("bench_mcfg_tts");
    let file_name_string = format!("{}{}", &file_name, "_bench_mcfg_tts.txt");

    let a = TreeStackAutomaton::from(grammar);

    let s1 = TTSElement::new();

    let rec = coarse_to_fine_recogniser!(a; s1);

    bench!(rec, file_name_string.clone(), sentences);
}

pub fn bench_mcfg_tts_rlb(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    equiv_rel : EquivalenceRelation<String, String>,
    sentences : Vec<String>
    ){
    println!("bench_mcfg_tts_rlb");
    let file_name_string = format!("{}{}", &file_name, "_bench_mcfg_tts_rlb.txt");

    let a = TreeStackAutomaton::from(grammar);

    let s1 = TTSElement::new();

    let f = |ps: & PosState<PMCFGRule<_,_,_,>>| ps.map(|r| r.map_nonterminals(|nt| equiv_rel.project(nt)));
    let s2 = RlbElement::new(&f);

    let rec = coarse_to_fine_recogniser!(a; s1, s2);

    bench!(rec, file_name_string.clone(), sentences);

}

pub fn bench_mcfg_tts_rlb_ptk(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    equiv_rel : EquivalenceRelation<String, String>,
    sentences : Vec<String>,
    ptk : usize
    ){

    println!("bench_mcfg_tts_rlb_ptk_{}", ptk);

    let file_name_string = format!("{}{}{}{}", &file_name, "_bench_mcfg_tts_rlb_ptk_", ptk, ".txt");

    let a = TreeStackAutomaton::from(grammar);

    let s1 = TTSElement::new();

    let f = |ps: & PosState<PMCFGRule<_,_,_,>>| ps.map(|r| r.map_nonterminals(|nt| equiv_rel.project(nt)));
    let s2 = RlbElement::new(&f);

    let s3 = PDTopKElement::new(ptk);

    let rec = coarse_to_fine_recogniser!(a; s1, s2, s3);

    bench!(rec, file_name_string.clone(), sentences);
    
}

pub fn bench_mcfg_tts_ptk(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    sentences : Vec<String>,
    ptk : usize
    ){
    
    println!("bench_mcfg_tts_ptk_{}", ptk);

    let file_name_string = format!("{}{}{}{}", &file_name, "_bench_mcfg_tts_ptk_", ptk, ".txt");

    let a = TreeStackAutomaton::from(grammar);

    let s1 = TTSElement::new();

    let s2 = PDTopKElement::new(ptk);

    let rec = coarse_to_fine_recogniser!(a; s1, s2);

    bench!(rec, file_name_string.clone(), sentences);
    
}

pub fn bench_mcfg_tts_ptk_rlb(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    equiv_rel : EquivalenceRelation<String, String>,
    sentences : Vec<String>,
    ptk : usize
    ){
    println!("bench_mcfg_tts_ptk_{}_rlb", ptk);

    let file_name_string = format!("{}{}{}{}", &file_name, "_bench_mcfg_tts_ptk_", ptk, "_rlb.txt");

    let a = TreeStackAutomaton::from(grammar);

    let s1 = TTSElement::new();

    let s2 = PDTopKElement::new(ptk);

    let f = |ps: & PosState<PMCFGRule<_,_,_,>>| ps.map(|r| r.map_nonterminals(|nt| equiv_rel.project(nt)));
    let s3 = RlbElement::new(&f);

    let rec = coarse_to_fine_recogniser!(a; s1, s2, s3);

    bench!(rec, file_name_string.clone(), sentences);
    
}

pub fn bench_mcfg_rlb(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    equiv_rel : EquivalenceRelation<String, String>,
    sentences : Vec<String>
    ){
    println!("bench_mcfg_rlb");
    let file_name_string = format!("{}{}", &file_name, "_bench_mcfg_rlb.txt");

    let a = TreeStackAutomaton::from(grammar);

    let f = |ps: & PosState<PMCFGRule<_,_,_,>>| ps.map(|r| r.map_nonterminals(|nt| equiv_rel.project(nt)));
    let s1 = RlbElementTSA::new(&f);

    let rec = coarse_to_fine_recogniser!(a; s1);

    bench!(rec, file_name_string.clone(), sentences);
}

pub fn bench_mcfg_rlb_tts(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    equiv_rel : EquivalenceRelation<String, String>,
    sentences : Vec<String>
    ){
    println!("bench_mcfg_rlb_tts");
    let file_name_string = format!("{}{}", &file_name, "_bench_mcfg_rlb_tts.txt");

    let a = TreeStackAutomaton::from(grammar);

    let f = |ps: & PosState<PMCFGRule<_,_,_,>>| ps.map(|r| r.map_nonterminals(|nt| equiv_rel.project(nt)));
    let s1 = RlbElementTSA::new(&f);

    let s2 = TTSElement::new();

    let rec = coarse_to_fine_recogniser!(a; s1, s2);

    bench!(rec, file_name_string.clone(), sentences);
}

pub fn bench_mcfg_rlb_tts_ptk(
    file_name : String,
    grammar : PMCFG<String, String, LogDomain<f64>>,
    equiv_rel : EquivalenceRelation<String, String>,
    sentences : Vec<String>,
    ptk : usize
    ){

    println!("bench_mcfg_rlb_tts_ptk_{}", ptk);

    let file_name_string = format!("{}{}{}{}", &file_name, "_bench_mcfg_rlb_tts_ptk_", ptk, ".txt");

    let a = TreeStackAutomaton::from(grammar);

    let f = |ps: & PosState<PMCFGRule<_,_,_,>>| ps.map(|r| r.map_nonterminals(|nt| equiv_rel.project(nt)));
    let s1 = RlbElementTSA::new(&f);

    let s2 = TTSElement::new();

    let s3 = PDTopKElement::new(ptk);

    let rec = coarse_to_fine_recogniser!(a; s1, s2, s3);

    bench!(rec, file_name_string.clone(), sentences);
    
}













