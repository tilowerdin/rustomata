use num_traits::Zero;
use std::marker::PhantomData;
use std::ops::AddAssign;
use std::ops::MulAssign;
use std::hash::Hash;
use num_traits::One;
use std::collections::HashSet;


use crate::automata::push_down_automaton::{PushDown,PushDownInstruction,PushDownAutomaton};
use crate::approximation::ApproximationStrategy;

use crate::recognisable::automaton::Automaton;
use crate::approximation::ApproximationInstance;
use crate::recognisable::Transition;



/// `ApproximationStrategy`that limits a `PushDownAutomaton` to a certain height.
#[derive(Clone, Debug)]
pub struct PDTopKElement<A> {
    _dummy: PhantomData<A>,
    pub size: usize,
}

impl<A> PDTopKElement<A> {
    pub fn new(size: usize) -> Self {
        assert!(size >= 1);
        PDTopKElement{
            _dummy: PhantomData,
            size: size,
        }
    }
}

impl<A, T, W> ApproximationStrategy<T, W> for PDTopKElement<A>
    where A: Clone + Ord + Hash,
          T: Clone + Eq + Hash + Ord,
          W: AddAssign + Copy + MulAssign + One + Ord + Zero,
{
    type I1 = PushDownInstruction<A>;
    type I2 = PushDownInstruction<A>;
    type A1 = PushDownAutomaton<A, T, W>;
    type A2 = PushDownAutomaton<A, T, W>;

    fn approximate_storage(&self, a: PushDown<A>) -> PushDown<A> {
        if a.iter().len() <= self.size {
            a
        } else {
            let new_empty = a.empty().clone();
            let mut new_elements: Vec<_> = a.iter().cloned().rev().take(self.size - 1).collect();
            new_elements.push(new_empty);
            new_elements.reverse();
            PushDown::from(new_elements)
        }
    }

    fn approximate_instruction(&self, instr: &PushDownInstruction<A>)
                               -> PushDownInstruction<A>
    {
        match *instr {
            PushDownInstruction::Replace { ref current_val, ref new_val }
            | PushDownInstruction::ReplaceK { ref current_val, ref new_val, .. } => {
                PushDownInstruction::ReplaceK {
                    current_val: current_val.clone(),
                    new_val: new_val.clone(),
                    limit: self.size,
                    // TODO possible_values get set in approximate_automaton
                    possible_values: Vec::new(),
                }
            },
        }
    }

    fn approximate_automaton(
        self,
        automaton1: &Self::A1,
    ) -> (Self::A2, ApproximationInstance<Self, T, W>) {
        let mut instance = ApproximationInstance::new(self);
        let transitions2: Vec<_> = automaton1
            .transitions()
            .map(|t| instance.approximate_transition(t.clone()))
            .collect();
        let initial2 = instance.approximate_storage(automaton1.initial());

        let mut possible_values_set : HashSet<A> = HashSet::new();

        for trans in &transitions2 {
            match trans.instruction {
                PushDownInstruction::ReplaceK {
                    ref current_val,
                    ref new_val,
                    limit,
                    ref possible_values,
                } => {
                    for val in current_val {
                        possible_values_set.insert(val.clone());
                    }
                    for val in new_val {
                        possible_values_set.insert(val.clone());
                    }
                },
                _ => ()
            }
        }

        
        let mut transitions3 = Vec::new();

        for mut trans in transitions2 {
            let old_trans = trans.clone();
            let new_trans = match trans.instruction {
                PushDownInstruction::ReplaceK {
                    current_val,
                    new_val,
                    limit,
                    ..
                } => Transition {
                        instruction: PushDownInstruction::ReplaceK {
                                    current_val,
                                    new_val,
                                    limit,
                                    possible_values: possible_values_set.iter().map(|val| val.clone()).collect()
                                    },
                        ..old_trans.clone()
                    },
                _ => trans.clone(),
            };
            transitions3.push(new_trans.clone());
            let get_trans = Transition {
                weight: W::one(),
                ..old_trans
            };
            let new_map_trans = Transition {
                weight: W::one(),
                ..new_trans
            };
            let reverse_transitions = instance.reverse_transition_map.get(&get_trans).unwrap();
            instance.reverse_transition_map.insert(new_map_trans, reverse_transitions.clone());
            instance.reverse_transition_map.remove(&get_trans);
        }

        (Self::A2::from_transitions(transitions3, initial2), instance)
    }
}

#[cfg(test)]
mod test {
    use crate::grammars::cfg::CFG;
    use log_domain::LogDomain;
    use crate::automata::push_down_automaton::PushDownAutomaton;
    use super::PDTopKElement;
    use crate::approximation::ApproximationStrategy;
    use crate::recognisable::Recognisable;

    fn get_grammar() -> CFG<String,String,LogDomain<f64>> {
        let r0_string = "S → [Nt A, Nt A, Nt A, Nt A, Nt A ] # 1";
        let r1_string = "A → [T a                         ] # 1";


        let mut g_string = String::from("initial: [S]\n\n");
        g_string.push_str(r0_string.clone());
        g_string.push_str("\n");
        g_string.push_str(r1_string.clone());

        let g: CFG<String, String, LogDomain<f64>> = g_string.parse().unwrap();

        g
    }

    #[test]
    fn test_topk() {
        //create (and test) initial push down automata
        let g = get_grammar();

        let a = PushDownAutomaton::from(g);

        let ptk = PDTopKElement::new(4);

        let (b, _) = ptk.approximate_automaton(&a);

        assert_eq!(None, a.recognise(vec!["a".to_string(), "a".to_string(), "a".to_string(), "a".to_string() ]).next());
        assert_ne!(None, b.recognise(vec!["a".to_string(), "a".to_string(), "a".to_string(), "a".to_string() ]).next());
        assert_ne!(None, a.recognise(vec!["a".to_string(), "a".to_string(), "a".to_string(), "a".to_string(), "a".to_string()]).next());
        assert_ne!(None, b.recognise(vec!["a".to_string(), "a".to_string(), "a".to_string(), "a".to_string(), "a".to_string()]).next());
        assert_ne!(None, b.recognise(vec!["a".to_string(), "a".to_string(), "a".to_string(), "a".to_string(), "a".to_string(), "a".to_string(), "a".to_string()]).next());
    }

    // #[test]
    // fn test_ptk_to_nfa(){
    //     let g: CFG<String, String, LogDomain<f64>>
    //         = "initial: [A]\n\
    //            \n\
    //            A → [T a, Nt A, T b]  # 0.6\n\
    //            A → []                # 0.4".parse().unwrap();

    //     let a = PushDownAutomaton::from(g);

    //     let ptk = PDTopKElement::new(3);

    //     let (b, _) = ptk.approximate_automaton(&a);

    //     let (nfa, _) = from_pd(&b).unwrap();

    //     assert_ne!(None, a.recognise(vec!["a".to_owned(), "b".to_owned()]).next());
    //     assert_eq!(None, a.recognise(vec!["a".to_owned(), "a".to_owned()]).next());
    //     assert_ne!(None, nfa.recognise(&vec!["a".to_owned(), "b".to_owned()]).next());
    //     assert_eq!(None, a.recognise(vec!["a".to_owned(), "a".to_owned(), "b".to_owned()]).next());
    //     assert_ne!(None, nfa.recognise(&vec!["a".to_owned(), "a".to_owned(), "b".to_owned()]).next());
    // }
}

