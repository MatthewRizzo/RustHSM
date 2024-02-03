use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Error};
use std::rc::Rc;

use crate::errors::{HSMError, HSMResult};
use crate::state::{StateBaseBehavior, StateId, EventBase, HSMControllerDecoratorBase};
///! Contains generic struct representing the context for a HSM.
/// Context will be composed of states that fulfill the state trait
// use crate::state::{BaseState, StateTree};
use crate::tree::{
    NodeDataConstraints, NodeOperations, Tree, TreeNode, TreeNodeDataRef, TreeNodeRef,
    TreeOperations,
};

/// Alias for the tree's used for states
pub type StateTree<T> = Tree<TreeNode<T>>;

/// Container of all state's in the StateMachine
/// Tree representing all state's in the HSM.
/// Where State is the datrastructure held by the tree's node(s)
pub struct BaseHSMController<State, EventEnum>
where
    State: StateBaseBehavior<EventEnum = EventEnum> + NodeDataConstraints + PartialEq,
{
    tree: Tree<TreeNode<State>>,
    current_state: Option<TreeNodeRef<State>>,
}

impl<'a, State, EventEnum> BaseHSMController<State, EventEnum>
where
    State: StateBaseBehavior<EventEnum = EventEnum> + NodeDataConstraints + PartialEq,
{
    // Create the HSM Context. Provide the data for the root node. Should
    // have all possible events to handle.
    // TODO - have new accept the full tree at construction
    // pub(crate) fn new(top_state:  Rc<RefCell<State>>) -> BaseHSMController<State> {
    //     let tree = Tree::create_tree(top_state);
    //     BaseHSMController {
    //         tree,
    //         current_state: None,
    //     }
    // }

    // todo - rework needing TreeNode in here
    pub(crate) fn new<EventEnums>(tree: StateTree<State>) -> BaseHSMController<State, EventEnum> {
        BaseHSMController {
            tree,
            current_state: None,
        }
    }

    /// Add a state to the HSM
    /// Return:
    /// The state's id within the HSM...you should hold onto all of them
    // pub fn add_state(&mut self,
    //     new_state_data: TreeNodeDataRef<State>,
    // ) -> HSMResult<StateId> {
    //     let x = new_state_data.borrow().get_node_parent();

    //     // let parent_node_id = match new_state_data.borrow().get_node_parent()
    //     // {
    //     //     None => 0,
    //     //     Some(parent_node) => {

    //     //     }
    //     // };
    //     let raw_state_id = self.tree.add_node(new_state_data, parent_state_id.id);

    //     Ok(StateId::new(raw_state_id))
    // }

    /// Initialize the StateMachine to a specific starting state
    pub fn init(&'a mut self, initial_state_id: StateId) -> HSMResult<()> {
        let initial_state_node = self
            .tree
            .get_node_by_id(initial_state_id.id)
            .ok_or_else(|| HSMError::GenericError("Invalid init state!".to_string()))?;
        self.current_state = Some(initial_state_node);
        Ok(())
    }

    /// API to dispatch an event into the HSM's context
    /// # Return
    /// * Error - If no state handles the event (you made a mistake somehow)
    /// * Success - If a state handles the event. Does not care how many state
    ///             changes occur.
    /// # Note
    /// * Error will also correspond to the event function returning false
    /// * Success will also correspond to the event function returning true
    pub fn dispatch_event<EventData>(
        &mut self,
        event: &dyn FnMut(EventData) -> bool,
    ) -> HSMResult<()> {
        self.handle_event(event)
    }

    /// Internal handler that handles events that all state's have implemented.
    /// Will change state as necessary until the event is handled by a state.
    /// # Return
    /// * Error - If no state handles the event (you made a mistake somehow)
    /// * Success - If a state handles the event. Does not care how many state
    ///             changes occur.
    /// # Note
    /// * Error will also correspond to the event function returning false
    /// * Success will also correspond to the event function returning true
    fn handle_event<EventData>(&mut self, event: &dyn FnMut(EventData) -> bool) -> HSMResult<()> {
        // some change state's will occur until we hit root or true is returned
        let mut handled = false;

        while !handled {
            // get the state's data
            let current_state_impl: TreeNodeRef<State> = self
                .current_state
                .as_ref()
                .borrow_mut()
                .ok_or_else(|| HSMError::EventNotImplemented("Unhandled Event".to_string()))?
                .clone();

            // let x = current_state_impl.into_inner().handle_event();
            // let state = current_state_impl.into_inner().get_node_data();
        }

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

    enum LightEvents {
        Toggle,
        TurnOn,
        TurnOff,
    }

    #[derive(PartialEq)]
    struct LightBaseState {}

    #[derive(PartialEq)]
    struct LightOn {}

    #[derive(PartialEq)]
    struct LightOff {}

    impl LightBaseState {
        fn new() -> Rc<RefCell<Self>> {
            let state = LightBaseState {};
            Rc::new(RefCell::new(state))
        }
    }

    impl NodeDataConstraints for LightBaseState {}
    impl StateBaseBehavior for LightBaseState {
        type EventEnum = LightEvents;
    }

    impl NodeDataConstraints for LightOn {}
    impl StateBaseBehavior for LightOn {
        type EventEnum = LightEvents;
    }

    impl NodeDataConstraints for LightOff {}
    impl StateBaseBehavior for LightOff {
        type EventEnum = LightEvents;
    }

    #[test]
    fn create_hsm() {
        let top_state = LightBaseState::new();
        let light_on_state = Rc::new(RefCell::new(LightOn {}));
        let light_off_state = Rc::new(RefCell::new(LightOff {}));

        let mut light_tree: Tree<TreeNode<LightBaseState>> = Tree::create_tree(top_state);
        // todo maybe we need dynamic dispatch here
        light_tree.add_node_with_parent_node(light_on_state, top_state);

        let light_hsm_controller = BaseHSMController::new::<LightEvents>(light_tree);

        // light_hsm_controller.add_state(new_state_data);
        assert!(true);
    }
}
