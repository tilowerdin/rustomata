use rustomata::automata::tree_stack_automaton::TreeStackAutomaton;
use rustomata::recognisable::automaton::Automaton;
use rustomata::recognisable::Item;
use rustomata::automata::tree_stack_automaton::PosState;
use rustomata::grammars::pmcfg::PMCFGRule;
use rustomata::approximation::equivalence_classes::EquivalenceRelation;
use log_domain::LogDomain;
use rustomata::grammars::pmcfg::PMCFG;
use rustomata::approximation::relabel::{RlbElement,RlbElementTSA};
use rustomata::approximation::tts::TTSElement;
use rustomata::approximation::ApproximationStrategy;
use rustomata::recognisable::Recognisable;

fn get_grammar_string() -> String {
    // creating the grammar string that accepts a^i b^j c^i d^j
    let mut grammar_string = String::new();
    grammar_string.push_str("initial: [S]\n");
    grammar_string.push_str("\n");
    grammar_string.push_str("S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\n");
    grammar_string.push_str("A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\n");
    grammar_string.push_str("A → [[],  []                             ] (    )   # 0.5\n");
    grammar_string.push_str("B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\n");
    grammar_string.push_str("B → [[],  []                             ] (    )   # 0.5\n");
    grammar_string
}

fn get_classes_string() -> String {
    // creating the equivalence classes from PMCFGRule to String
    let mut classes_string = String::new();
    classes_string.push_str("S [\"S → [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B)\"]\n");
    classes_string.push_str("A [\"A → [[T a, Var 0 0],  [T c, Var 0 1]     ] (A   )   # 0.5\", \"A → [[],  []] (    )   # 0.5\"]\n");
    classes_string.push_str("B [\"B → [[T b, Var 0 0],  [T d, Var 0 1]     ] (B   )   # 0.5\", \"B → [[],  []] (    )   # 0.5\"]\n");
    classes_string.push_str("R *\n");
    classes_string
}

fn get_accepting_input() -> Vec<String> {
    vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]
}

#[test]
fn test_unapproximate_runs() {
    
    let grammar_string = get_grammar_string();
    let classes_string = get_classes_string();
    

    // creating an accepting input vector
    let accepting_input = get_accepting_input();

    // parse the grammar into a PMCFG
    let g : PMCFG<String,String,LogDomain<f64>> = grammar_string.parse().unwrap();

    // create the Tree Stack Automaton from the grammar
    let a = TreeStackAutomaton::from(g);

    // create the first approximation strategy which approximates the TSA into a Push Down Automaton
    let tts = TTSElement::new();

    // approximate the TSA into a PDA
    let (b,strat1) = tts.approximate_automaton(&a);

    // create the second approximation strategy by first create an Equivalence Relation, then a mapping and finally the relabel strategy
    let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = classes_string.parse().unwrap();
    let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));
    let rlb = RlbElement::new(&f);

    // approximate the PDA into another PDA using the relabel strategy
    let (c,strat2) = rlb.approximate_automaton(&b);

    // recognise the accepting input by the approximated PDA
    let recs_c = c.recognise(accepting_input.clone());

    let mut got_valid_run = false;

    // try to unapproximate the runs and get one run of the original TSA that is accepted by the original TSA
    for Item(_,run_c) in recs_c {
        let unapproxs2 = strat2.unapproximate_run(run_c);
        for unapprox2 in unapproxs2 {
            let checked_bs = b.check_run(unapprox2);
            for Item(_,checked_b) in checked_bs {
                let unapproxs1 = strat1.unapproximate_run(checked_b);
                for unapprox1 in unapproxs1 {
                    let checked_as = a.check_run(unapprox1);
                    for Item(_,checked_a) in checked_as {
                        got_valid_run = true;
                    }
                }
            }
        }
    }

    // unapproximating the runs should result in one accepting run of the original TSA since the input is an accepting one
    assert!(got_valid_run);
}

#[test]
fn test_collecting_weights_of_similar_transitions() {
    let grammar_string = get_grammar_string();
    let classes_string = get_classes_string();
    let accepting_input = get_accepting_input();

    let g : PMCFG<String, String,LogDomain<f64>> = grammar_string.parse().unwrap();

    let a = TreeStackAutomaton::from(g);

    let e: EquivalenceRelation<PMCFGRule<_,_,_>, String> = classes_string.parse().unwrap();
    let f = |ps: &PosState<_>| ps.map(|nt| e.project(nt));
    let rlb = RlbElementTSA::new(&f);

    let (b,_) = rlb.approximate_automaton(&a);

    let recs = b.recognise(accepting_input);

    let mut count = 0;

    for _ in recs {
        count += 1;
    }

    assert_eq!(1, count);
}