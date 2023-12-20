use std::fmt::{Debug, Error};

use crate::errors::{HSMError, HSMResult};
use crate::state::StateBaseBehavior;
///! Contains generic struct representing the context for a HSM.
/// Context will be composed of states that fulfill the state trait
// use crate::state::{BaseState, StateTree};
use crate::tree::{NodeDataConstraints, NodeOperations, Tree, TreeNode, TreeOperations};

/// Container of all state's in the StateMachine
/// Tree representing all state's in the HSM.
/// Where State is the datrastructure held by the tree's node(s)
pub struct HSMContext<'a, State>
where
    State: StateBaseBehavior + NodeDataConstraints + PartialEq,
{
    tree: Tree<TreeNode<'a, State>>,
    current_state: Option<&'a TreeNode<'a, State>>,
}

impl<'a, State> HSMContext<'a, State>
where
    State: StateBaseBehavior + NodeDataConstraints + PartialEq + Debug,
{
    /// Create the HSM Context. Provide the data for the root node. Should
    /// have all possible events to handle.
    pub(crate) fn new(top_state: &'a State) -> HSMContext<'a, State> {
        let tree = Tree::create_tree(top_state);
        HSMContext {
            tree,
            current_state: None,
        }
    }

    /// Add a state to the HSM
    /// Return:
    /// The state's id within the HSM...you should hold onto all of them
    pub fn add_state(&mut self, new_state: &'a State, parent_state: &'a State) -> HSMResult<u16> {
        let parent_node = self.tree.find_node_by_data(parent_state).ok_or_else(|| {
            HSMError::GenericError(format!(
                "Error finding parent node containing data {:?}",
                parent_state
            ))
        })?;

        let root_node = self.tree.get_root_node();

        let new_node = TreeNode::new(new_state, Some(parent_node), Some(root_node));
        let state_id = self.tree.add_node(new_node);

        Ok(state_id)
    }

    /// Initialize the StateMachine to a specific starting state
    pub fn init(&'a mut self, initial_state_id: u16) {
        let initial_state_node = self.tree.get_node_by_id(initial_state_id);
        self.current_state = Some(initial_state_node);
    }

    /// API to dispatch an event into the HSM's context
    pub fn dispatch(&self) -> HSMResult<bool> {
        todo!();
    }

    /// Internal handler that maps the event id to the handler that all state's
    /// have implemented
    fn handle(&self, event: u32) -> HSMResult<bool> {
        // some change state's will occur until we hit root or true is returned
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // trait LightBaseState {
    //     type ParentState = BaseState;
    //     pub fn toggle_switch(&self, _: &Self::ParentState) -> bool;
    // }

    // struct LightOn {
    //     num_times_turned_on: u32,
    //     num_times_turned_off: u32,
    // }

    // impl LightBaseState for LightOn {
    //     type BaseState = State;
    //     fn handle_state_enter(&mut self) {
    //         num_times_turned_on += 1;
    //     }
    //     fn handle_state_exit(&mut self) {
    //         num_times_turned_off += 1;
    //     }
    //     pub fn toggle_switch(&self, _: &Self::ParentState) -> bool {}
    // }

    // impl LightOn {
    //     fn new() -> LightOn {
    //         LightOn {
    //             num_times_turned_on: 0,
    //             num_times_turned_off: 0,
    //         }
    //     }
    // }

    // struct LightOff {
    //     num_times_turned_on: u32,
    //     num_times_turned_off: u32,
    // }

    // impl State for LightOff {
    //     fn handle_state_enter(&mut self) {
    //         num_times_turned_on += 1;
    //     }
    //     fn handle_state_exit(&mut self) {
    //         num_times_turned_off += 1;
    //     }
    // }

    // impl LightOff {
    //     fn new() -> LightOff {
    //         LightOff {
    //             num_times_turned_on: 0,
    //             num_times_turned_off: 0,
    //         }
    //     }
    // }

    // fn create_state_tree() -> StateTree

    #[test]
    fn create_hsm() {
        // HSMContext::new(state_tree);
        assert!(true);
    }
}
