///! This file contains the logic for a state engine comprised of many
///! composable states
use crate::{
    errors::{HSMError, HSMResult},
    events::DecoratableEventBase,
    state::{ComposableStateBase, RefStates, StateId},
};
use std::{cell::RefCell, rc::Rc};

/// Compose / decorate your hsm controller with this
pub struct DecoratableHSMControllerBase {
    /// We own the vector of states, but the states themselves are owned by others
    states: RefStates,
    current_state: Rc<RefCell<ComposableStateBase>>,
    /// Only set during handle_event if there is a change_state
    requested_new_state: Option<StateId>,
    /// Used to cache the current known sequence of events
    state_change_string: String,
}

impl DecoratableHSMControllerBase {
    pub fn new(top_state: Rc<RefCell<ComposableStateBase>>) -> Self {
        DecoratableHSMControllerBase {
            states: vec![top_state.clone()],
            current_state: top_state,
            requested_new_state: None,
            state_change_string: String::new(),
        }
    }

    pub fn init(&mut self, initial_state: StateId) -> HSMResult<()> {
        // if *initial_state.get_id() as usize >= self.states.as_ref().borrow().len() {
        if *initial_state.get_id() as usize >= self.states.len() {
            return Err(HSMError::InvalidStateId(format!(
                "Initial State with Id {} is not valid. There are only {} states!",
                initial_state.get_id(),
                self.states.len() - 1
            )));
        }

        let new_current_state: Rc<RefCell<ComposableStateBase>>;
        {
            new_current_state = Self::get_state_by_id(&self.states, &initial_state)
            // new_current_state = self.get_state_by_id(initial_state);
        }
        self.current_state = new_current_state;

        Ok(())
    }

    /// Changes the state. Will IMMEDIATELY go to the LCA state, but will NOT
    /// go to target state until AFTER the handling of the current event that
    /// triggered the change_state
    pub fn change_state(&mut self, target_state_id: StateId) {
        let target_state_name = self.get_state_name(&target_state_id);

        assert!(
            target_state_name.is_some(),
            "change_state: Target state provided with id ({}) is invalid!",
            target_state_id.get_id()
        );
        debug_assert!(
            self.requested_new_state.is_none(),
            "change_state: A new state cannot be requested if another already is!"
        );

        let current_state = self.current_state.clone();
        let current_state_id = current_state.as_ref().borrow().get_state_id().to_owned();
        let current_state_name = self.get_state_name(&current_state_id).unwrap();

        self.state_change_string.push_str(
            format!(
                "[change_state({} >> {} )",
                current_state_name,
                target_state_name.unwrap()
            )
            .as_str(),
        );

        if self.requested_new_state.is_none() {
            println!(
                "{}] - ILLEGAL NESTED STATE CHANGE IGNORED!",
                self.state_change_string
            );
            return;
        }

        let target_state = Self::get_state_by_id(&self.states, &target_state_id);
        let lca_state_id = Self::find_lca(current_state, target_state)
            .expect(format!("Error finding lca for {} ", self.state_change_string).as_str());

        self.exit_states_until_target(lca_state_id);

        // Alerts handle_change_state to do work at the end of handle
        self.requested_new_state = Some(target_state_id);
    }

    pub fn handle_event(&mut self, event: &DecoratableEventBase) {
        // keep going until event is handled (true) or we reach the end
        let mut current_state = self.current_state.clone();
        loop {
            // let next_state = current_state.as_ref().borrow().parent_state.clone();
            let next_state =
                ComposableStateBase::get_parent_state(&current_state.as_ref().borrow());
            if next_state.is_none() {
                break;
            }

            let is_handled =
                ComposableStateBase::handle_event(&current_state.as_ref().borrow(), event);

            if is_handled {
                // event has been handled!
                break;
            }

            // See if parent state handles this
            current_state = next_state.unwrap();
        }
        self.handle_state_change();
    }

    pub fn get_state_name(&self, state_id: &StateId) -> Option<String> {
        for state_ref in &self.states {
            let borrowed_state = state_ref.as_ref().borrow();
            if borrowed_state.get_state_id() == state_id {
                return Some(borrowed_state.get_state_name().clone());
            }
        }
        None
    }

    /// Precondition - the state id is valid!
    /// todo - make this self if possible
    fn get_state_by_id(states: &RefStates, state_id: &StateId) -> Rc<RefCell<ComposableStateBase>> {
        states.get(*state_id.get_id() as usize).unwrap().clone()
    }

    /// Check if a change state was enqueued while processing an event.
    /// Given that change_state exits all states up-to-but-not-including lca,
    /// enter the LCA state and then all sub-state's (including the target state).
    /// Only then do we START the target state
    fn handle_state_change(&mut self) {
        if self.requested_new_state.is_none() {
            return;
        }

        let target_state =
            Self::get_state_by_id(&self.states, &self.requested_new_state.clone().unwrap());
        let target_to_lca_path = target_state.as_ref().borrow().get_path_to_root();

        self.state_change_string.push_str("[");

        for state_id_to_enter in target_to_lca_path.into_iter().rev() {
            let state_to_enter = Self::get_state_by_id(&self.states, &state_id_to_enter);
            state_to_enter.as_ref().borrow_mut().handle_state_enter();

            let state_to_enter_name = state_to_enter.as_ref().borrow().get_state_name().clone();
            self.state_change_string
                .push_str(format!("{}(ENTER)", state_to_enter_name).as_str());
            // self.state_change_string.push_str(format!("{}(START)", state_to_enter.as_ref().borrow().get_state_name()).as_str());
        }

        // Start the target state!
        target_state.as_ref().borrow_mut().handle_state_start();
        let target_state_name = target_state.as_ref().borrow().get_state_name().clone();
        self.state_change_string
            .push_str(format!("{}(START)]", target_state_name).as_str());

        // todo add tracing logic showing the full path!

        // Log the current chain and reset the message
        println!("{}", self.state_change_string);
        self.state_change_string = String::new();
        self.requested_new_state = None;
    }

    /// get LCA between current state and other state
    /// todo - unit test this. make this a self function
    fn find_lca(
        source_state: Rc<RefCell<ComposableStateBase>>,
        target_state: Rc<RefCell<ComposableStateBase>>,
    ) -> HSMResult<StateId> {
        let target_state_id = target_state.as_ref().borrow().get_state_id().clone();

        if target_state_id.get_id() == &StateId::get_top_state_id() {
            return Err(HSMError::InvalidStateId(
                "There is no LCA between a state and Top!!!".to_string(),
            ));
        }

        let source_path_to_root = source_state.as_ref().borrow().get_path_to_root();
        let target_path_to_root = target_state.as_ref().borrow().get_path_to_root();
        let mut root_to_source_path = source_path_to_root.clone();
        root_to_source_path.reverse();
        let mut root_to_target_path = target_path_to_root.clone();
        root_to_target_path.reverse();

        // Compare the two paths, starting from the ends of the paths (where the root is)
        // keep going until the nodes diverge. The last node before the paths diverge is the LCA.
        let mut last_known_common_state = StateId::get_top_state_id();
        for root_to_source_state in root_to_source_path {
            for root_to_target_state in root_to_target_path.clone() {
                if root_to_source_state.get_id() == root_to_target_state.get_id() {
                    last_known_common_state = *root_to_source_state.get_id();
                } else {
                    break;
                }
            }
        }

        Ok(StateId::new(last_known_common_state))
    }

    /// Exits all states along the path to target (not including target)
    fn exit_states_until_target(&mut self, target_state_id: StateId) {
        let mut current_state = self.current_state.clone();

        loop {
            let opt_parent_state =
                ComposableStateBase::get_parent_state(&current_state.as_ref().borrow());
            // this should only happen if we reach top
            if opt_parent_state.is_none() {
                break;
            }

            if current_state.as_ref().borrow().get_state_id() == &target_state_id {
                break;
            }

            current_state.as_ref().borrow_mut().handle_state_exit();

            current_state = opt_parent_state.unwrap();
        }
    }
}
