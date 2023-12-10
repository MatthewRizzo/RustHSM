use crate::tree::{FindDataNode, Tree, TreeNode};
///! Contains generic struct representing a valid state / its traits
use std::collections::VecDeque;

pub trait BaseState {
    fn handle_state_enter(&self);
    fn handle_state_exit(&self);
}

trait ChangeState<StateImpl> {
    type T;
    // Change state to the state with the provided id
    fn change_state(&mut self, state_id: u32);
}

/// States to traverse through to arrive at the end state
struct StateTraversal<'a, StateImpl>
where
    StateImpl: BaseState,
{
    transition_states_stack: VecDeque<&'a StateNode<'a, StateImpl>>,
    end_state: Option<&'a StateNode<'a, StateImpl>>,
}

impl<'a, StateImpl> StateTraversal<'a, StateImpl>
where
    StateImpl: BaseState,
{
    fn new(
        transition_states_stack: VecDeque<&'a StateNode<'a, StateImpl>>,
        end_state: Option<&'a StateNode<'a, StateImpl>>,
    ) -> StateTraversal<'a, StateImpl> {
        StateTraversal {
            transition_states_stack,
            end_state,
        }
    }
}

// TreeNode
// todo - turn the state node into a proper tree node
// todo - integrate tree node and state node together with the proper traits
// todo - plumb the add state call to add node + hierarchy
struct StateData<'a> {
    state_id: u32,
}

struct StateNode<'a, StateImpl>
where
    StateImpl: BaseState,
{
    state_id: u32,
    current_state: StateImpl,
    children: Vec<StateNode<'a, StateImpl>>,
    parent: Option<&'a StateNode<'a, StateImpl>>,
}

impl<'a, StateImpl> StateNode<'a, StateImpl>
where
    StateImpl: BaseState,
{
    fn new(state_id: &u32, current_state: StateImpl, parent: &StateNode<'a, StateImpl>) -> Self {
        StateNode {
            state_id: state_id.to_owned(),
            current_state,
            children: Vec::new(),
            parent: Some(parent),
        }
    }

    fn get_root_node(&self) -> &StateNode<'a, StateImpl> {
        let mut previous_node = self;
        let mut current_node = Some(self);
        while current_node.is_some() {
            previous_node = current_node.unwrap();
            current_node = current_node.unwrap().parent;
        }

        previous_node
    }

    fn traverse_to_state(&self, state_id: u32) -> Option<StateTraversal<'a, StateImpl>> {
        let mut path_stack: Stack<StateNode<StateImpl>> = Stack::new();

        if self.state_id == state_id {
            return Some(StateTraversal::new(path_stack, Some(&self)));
        }

        let root_node: &StateNode<'_, StateImpl> = self.get_root_node();
        root_node.depth_first_search(state_id)
    }
}

impl<'a, StateImpl> ChangeState<StateImpl> for StateNode<'a, StateImpl>
where
    StateImpl: BaseState,
{
    type T = StateNode<'a, StateImpl>;
    fn change_state(&mut self, state_id: u32) {
        if self.state_id == state_id {
            self.current_state.handle_state_exit();
            self.current_state.handle_state_enter();
        }
        let path_to_node = self.traverse_to_state(state_id);

        // letstate.traverse_to_state(state_id);
    }
}
