use num_traits::{One,Zero};
use std::collections::BinaryHeap;
use std::ops::{MulAssign,AddAssign};
use std::rc::Rc;
use std::hash::Hash;

use crate::approximation::{ApproximationInstance, ApproximationStrategy};
use crate::recognisable::automaton::Automaton;
use crate::recognisable::{Instruction, Item, Recognisable};
use search::agenda::weighted::Weighted;

pub struct CoarseToFineRecogniser<Rec, SubRec, Strategy, T, W>
where
    Rec: Automaton<T, W>,
    SubRec: Recognisable<
        T,
        W,
        Parse = Item<<Strategy::I2 as Instruction>::Storage, Strategy::I2, T, W>,
    >,
    Strategy: ApproximationStrategy<T, W> + Sized,
    Strategy::I1: Instruction,
    Strategy::I2: Instruction,
    T: Clone + Eq + Ord + Hash,
    W: Clone + MulAssign + One + Ord + Zero + AddAssign,
{
    pub recogniser: Rc<Rec>,
    pub sublevel: Rc<SubRec>,
    pub approximation_instance: Rc<ApproximationInstance<Strategy, T, W>>,
}

struct CoarseToFineParseForest<'a, Rec, Strategy, T, W>
where
    Rec: Automaton<T, W>,
    Strategy: 'a + ApproximationStrategy<T, W>,
    Strategy::I1: Instruction,
    T: 'a + Clone + Eq + Ord + Hash,
    W: 'a + Clone + MulAssign + One + Ord + Zero + AddAssign,
{
    sublevel_parses:
        Box<Iterator<Item = Item<<Strategy::I2 as Instruction>::Storage, Strategy::I2, T, W>> + 'a>,
    recogniser: Rc<Rec>,
    approximation_instance: Rc<ApproximationInstance<Strategy, T, W>>,
    input_buffer: Option<Option<Item<<Strategy::I2 as Instruction>::Storage, Strategy::I2, T, W>>>,
    output_buffer: BinaryHeap<Item<<Strategy::I1 as Instruction>::Storage, Strategy::I1, T, W>>,
}

impl<'a, Rec, Strategy, T, W> CoarseToFineParseForest<'a, Rec, Strategy, T, W>
where
    Rec: Automaton<T, W>,
    Strategy: 'a + ApproximationStrategy<T, W>,
    Strategy::I1: Instruction,
    T: 'a + Clone + Eq + Ord + Hash,
    W: 'a + Clone + MulAssign + One + Ord + Zero + AddAssign,
{
    fn peek_input(
        &mut self,
    ) -> Option<&Item<<Strategy::I2 as Instruction>::Storage, Strategy::I2, T, W>> {
        if self.input_buffer.is_none() {
            self.input_buffer = Some(self.sublevel_parses.next());
        }
        match self.input_buffer {
            Some(Some(ref item)) => Some(item),
            Some(None) => None,
            _ => unreachable!(),
        }
    }

    fn next_input(
        &mut self,
    ) -> Option<Item<<Strategy::I2 as Instruction>::Storage, Strategy::I2, T, W>> {
        if self.input_buffer.is_none() {
            self.input_buffer = Some(self.sublevel_parses.next());
        }
        match self.input_buffer {
            Some(ref mut x) => x.take(),
            _ => unreachable!(),
        }
    }
}

impl<'a, Rec, Strategy, T, W> Iterator for CoarseToFineParseForest<'a, Rec, Strategy, T, W>
where
    Rec: Automaton<T, W, I = Strategy::I1>,
    Strategy: ApproximationStrategy<T, W>,
    Strategy::I1: Instruction + Ord,
    <Strategy::I1 as Instruction>::Storage: Ord,
    T: Clone + Eq + Ord + Hash,
    W: Clone + MulAssign + One + Ord + Zero + AddAssign,
{
    type Item = Item<<Strategy::I1 as Instruction>::Storage, Strategy::I1, T, W>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.output_buffer.is_empty()
            || self.peek_input().is_some()
                && self.output_buffer.peek().unwrap().get_weight()
                    < self.peek_input().unwrap().get_weight()
        {
            if let Some(Item(_, r2)) = self.next_input() {
                let r1s = self.approximation_instance.unapproximate_run(r2);
                for r1 in r1s {
                    for i1 in self.recogniser.check_run(r1) {
                        self.output_buffer.push(i1);
                    }
                }
            } else {
                return None;
            }
        }
        self.output_buffer.pop()
    }
}

impl<Rec, SubRec, Strategy, T, W> Recognisable<T, W>
    for CoarseToFineRecogniser<Rec, SubRec, Strategy, T, W>
where
    Rec: Automaton<T, W, I = Strategy::I1>,
    SubRec: Recognisable<
        T,
        W,
        Parse = Item<<Strategy::I2 as Instruction>::Storage, Strategy::I2, T, W>,
    >,
    Strategy: ApproximationStrategy<T, W>,
    Strategy::I1: Instruction + Ord,
    <Strategy::I1 as Instruction>::Storage: Ord,
    T: Clone + Eq + Ord + Hash,
    W: Clone + MulAssign + One + Ord + Zero + AddAssign,
{
    type Parse = Item<<Strategy::I1 as Instruction>::Storage, Strategy::I1, T, W>;

    fn recognise<'a>(&'a self, word: Vec<T>) -> Box<Iterator<Item = Self::Parse> + 'a> {
        Box::new(CoarseToFineParseForest {
            sublevel_parses: self.sublevel.recognise(word),
            recogniser: self.recogniser.clone(),
            approximation_instance: self.approximation_instance.clone(),
            input_buffer: None,
            output_buffer: BinaryHeap::new(),
        })
    }

    fn recognise_beam_search<'a>(
        &'a self,
        _: usize,
        _: Vec<T>,
    ) -> Box<Iterator<Item = Self::Parse> + 'a> {
        unimplemented!()
    }
}

#[macro_export]
macro_rules! coarse_to_fine_recogniser {
    ( $automaton:expr; $strategy:expr ) => {
        {
            let (aut0, strat_instance) = $strategy.approximate_automaton(&$automaton);
            CoarseToFineRecogniser {
                sublevel: Rc::new(aut0),
                recogniser: Rc::new($automaton),
                approximation_instance: Rc::new(strat_instance),
            }
        }
    };
    ( $automaton:expr; $strategy:expr, $( $strats:expr ),* ) => {
        {
            let (aut0, strat_instance) = $strategy.approximate_automaton(&$automaton);
            CoarseToFineRecogniser {
                sublevel: Rc::new(coarse_to_fine_recogniser!(aut0; $( $strats ),*)),
                recogniser: Rc::new($automaton),
                approximation_instance: Rc::new(strat_instance),
            }
        }
    };
    ( $automaton:expr, $strategies:expr ) => {
        {
            let tts_string = "tts".to_string();
            let ptk_string = "ptk".to_string();
            let rlb_string = "rlb".to_string();
            match $strategies.pop() { 
                Some((tts_string, _)) => {
                    let s1 = TTSElement::new();
                    match $strategies.pop() {
                        None => coarse_to_fine_recogniser!($automaton; s1),
                        Some((rlb_string, rlb_file)) => {
                            
                        }
                        Some(_) => panic!("Not implemented yet!"),
                    }
                },
                _ => panic!("Not implemented yet!"),
            }
        }
    }
}
