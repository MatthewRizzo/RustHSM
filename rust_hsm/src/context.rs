///! Contains generic struct representing the context for a HSM.
/// Context will be composed of states that fulfill the state trait
use crate::state::{BaseState, StateTree};

/// Container of all state's in the StateMachine
pub struct HSMContext<StateImpl>
where
    StateImpl: State,
{
    state_tree: StateTree<StateImpl>,
}

impl<StateImpl> HSMContext<StateImpl>
where
    StateImpl: State,
{
    pub fn new(state_tree: StateTree<StateImpl>) -> HSMContext<StateImpl> {
        HSMContext { state_tree }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait LightBaseState {
        type ParentState = BaseState;
        pub fn toggle_switch(&self, _: &Self::ParentState) -> bool;
    }

    struct LightOn {
        num_times_turned_on: u32,
        num_times_turned_off: u32,
    }

    impl LightBaseState for LightOn {
        type BaseState = State;
        fn handle_state_enter(&mut self) {
            num_times_turned_on += 1;
        }
        fn handle_state_exit(&mut self) {
            num_times_turned_off += 1;
        }
        pub fn toggle_switch(&self, _: &Self::ParentState) -> bool {}
    }

    impl LightOn {
        fn new() -> LightOn {
            LightOn {
                num_times_turned_on: 0,
                num_times_turned_off: 0,
            }
        }
    }

    struct LightOff {
        num_times_turned_on: u32,
        num_times_turned_off: u32,
    }

    impl State for LightOff {
        fn handle_state_enter(&mut self) {
            num_times_turned_on += 1;
        }
        fn handle_state_exit(&mut self) {
            num_times_turned_off += 1;
        }
    }

    impl LightOff {
        fn new() -> LightOff {
            LightOff {
                num_times_turned_on: 0,
                num_times_turned_off: 0,
            }
        }
    }

    // fn create_state_tree() -> StateTree

    #[test]
    fn create_hsm() {
        HSMContext::new(state_tree);
    }
}
