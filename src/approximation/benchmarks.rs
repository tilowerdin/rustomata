
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
use crate::approximation::ApproximationStrategy;
use crate::recognisable::Recognisable;
use crate::coarse_to_fine_recogniser;
use std::rc::Rc;
use crate::recognisable::coarse_to_fine::CoarseToFineRecogniser;

extern crate test;

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