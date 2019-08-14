use num_traits::Zero;
use std::hash::Hash;
use std::ops::{AddAssign, MulAssign};

use crate::approximation::*;
use crate::automata::push_down_automaton::*;

use crate::automata::tree_stack_automaton::tree_stack_instruction::TreeStackInstruction;
use crate::automata::tree_stack_automaton::TreeStackAutomaton;
use crate::automata::tree_stack_automaton::tree_stack::TreeStack;
use crate::automata::tree_stack_automaton::from_pmcfg::PosState;

/// `ApproximationStrategy` that uses the `Relabel` trait to relabel internal values via an `EquivalenceClass`
pub struct RlbElement<'a, A1, A2>
where
    A1: 'a,
    A2: 'a,
{
    pub mapping: &'a Fn(&A1) -> A2,
}

impl<'a, A1, A2> RlbElement<'a, A1, A2> {
    pub fn new(mapping: &'a Fn(&A1) -> A2) -> Self {
        RlbElement { mapping }
    }
}

impl<'a, A1, A2, T, W> ApproximationStrategy<T, W> for RlbElement<'a, A1, A2>
where
    A1: Clone + Hash + Ord,
    A2: Clone + Hash + Ord,
    T: Clone + Eq + Hash + Ord,
    W: AddAssign + Copy + MulAssign + One + Ord + Zero,
{
    type I1 = PushDownInstruction<A1>;
    type I2 = PushDownInstruction<A2>;
    type A1 = PushDownAutomaton<A1, T, W>;
    type A2 = PushDownAutomaton<A2, T, W>;

    fn approximate_storage(&self, pd: PushDown<A1>) -> PushDown<A2> {
        pd.map(&self.mapping)
    }

    fn approximate_instruction(&self, instr: &PushDownInstruction<A1>) -> PushDownInstruction<A2> {
        match *instr {
            PushDownInstruction::Replace {
                ref current_val,
                ref new_val,
            } => PushDownInstruction::Replace {
                current_val: current_val.iter().map(self.mapping).collect(),
                new_val: new_val.iter().map(self.mapping).collect(),
            },
        }
    }
}

pub struct RlbElementTSA<'a, A1, A2>
{
    pub mapping: &'a Fn(&A1) -> A2,
}

impl<'a, A1, A2> RlbElementTSA<'a, A1, A2> {
    pub fn new(mapping: &'a Fn(&A1) -> A2) -> Self {
        RlbElementTSA { mapping }
    }
}

impl<A1, A2, T, W> ApproximationStrategy<T, W> for RlbElementTSA<'_, A1, A2>
where
    A1: Clone + Hash + Ord,
    A2: Clone + Hash + Ord,
    T: Clone + Eq + Hash + Ord,
    W: AddAssign + Copy + MulAssign + One + Ord + Zero,
{
    type I1 = TreeStackInstruction<A1>;
    type I2 = TreeStackInstruction<A2>;
    type A1 = TreeStackAutomaton<A1, T, W>;
    type A2 = TreeStackAutomaton<A2, T, W>;

    fn approximate_storage(&self, ts: TreeStack<A1>) -> TreeStack<A2> {
        ts.map(& (self.mapping))
    }

    fn approximate_instruction(&self, instr: &TreeStackInstruction<A1>) -> TreeStackInstruction<A2> {
        instr.map(self.mapping)
    }
}

// impl<'a, A1, A2, T, W> ApproximationStrategy<T, W> for RlbElement<'a, A1, A2>
// where
//     A1: Clone + Hash + Ord,
//     A2: Clone + Hash + Ord,
//     T: Clone + Eq + Hash + Ord,
//     W: AddAssign + Copy + MulAssign + One + Ord + Zero,
// {
//     // type I1 = TreeStackInstruction<A1>;
//     // type I2 = TreeStackInstruction<A2>;
//     // type A1 = TreeStackAutomaton<A1, T, W>;
//     // type A2 = TreeStackAutomaton<A2, T, W>;

//     fn approximate_storage(&self, ts: TreeStack<A1>) -> TreeStack<A2> {
//         ts.map(&self.mapping)
//     }

//     fn approximate_instruction(&self, instr: &TreeStackInstruction<A1>) -> TreeStackInstruction<A2> {
//         instr.map(&self.mapping)
//     }
// }

#[cfg(test)]
mod tests {
    use super::super::equivalence_classes::EquivalenceRelation;
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_approximate_storage() {
        let rel = EquivalenceRelation::from_str("0 [0, 1]\n1 [2, 3]\n2 *").unwrap();
        let mapping = |ps: &PushState<_, _>| ps.map(|nt| rel.project(nt));
        let rlb = RlbElement::new(&mapping);

        let pushdown = PushDown::from(vec![
            PushState::Initial,
            PushState::Nt(0),
            PushState::Nt(1),
            PushState::T('a'),
            PushState::Nt(2),
            PushState::Nt(3),
            PushState::Nt(4),
            PushState::Designated,
        ]);
        let control_pushdown = PushDown::from(vec![
            PushState::Initial,
            PushState::Nt(0),
            PushState::Nt(0),
            PushState::T('a'),
            PushState::Nt(1),
            PushState::Nt(1),
            PushState::Nt(2),
            PushState::Designated,
        ]);

        assert_eq!(
            control_pushdown,
            <RlbElement<_, _> as ApproximationStrategy<char, u8>>::approximate_storage(
                &rlb, pushdown
            )
        );
    }

    #[test]
    fn test_approximate_instruction() {
        let rel = EquivalenceRelation::from_str("0 [0, 1]\n1 [2, 3]\n2 *").unwrap();
        let mapping = |ps: &PushState<_, _>| ps.map(|nt| rel.project(nt));
        let rlb = RlbElement::new(&mapping);

        let instruction = PushDownInstruction::Replace {
            current_val: vec![PushState::T('a'), PushState::Nt(4)],
            new_val: vec![PushState::T('b'), PushState::Nt(3)],
        };
        let control_instruction = PushDownInstruction::Replace {
            current_val: vec![PushState::T('a'), PushState::Nt(2)],
            new_val: vec![PushState::T('b'), PushState::Nt(1)],
        };

        assert_eq!(
            control_instruction,
            <RlbElement<_, _> as ApproximationStrategy<char, u8>>::approximate_instruction(
                &rlb,
                &instruction,
            )
        );
    }

    #[test]
    fn test_approx_ts_instruction() {
        // let rel = EquivalenceRelation::from_str("0 [0, 1]\n1 [2, 3]\n2 *").unwrap();
        // let mapping = |ps: &PosState<_>| ps.map(|nt| rel.project(nt));
        // let rlb = RlbElement::new(&mapping);

        // let intr_up = TreeStackInstruction::Up {
        //     n : 0,
        //     current_val : 
        // }
    }
}
