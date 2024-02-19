use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventsIF,
    state::{StateId, StateRef, StatesRefVec},
};

use std::{
    cell::RefCell,
    rc::Rc,
};

pub type HsmControllerRef = Rc<RefCell<dyn HsmController>>;

/// The traits required to be a proper HSM controller
/// Everything is implemented for consumers.
/// The rest is implemented by DecoratableHSMControllerBase.
/// No need to override.
/// Used to allow indirection between states and controller.
/// # Functions to implement that are trivial (if done right):
///     * add_state
///     * get_current_state
///     * set_current_state
///     * get_requested_new_state
///     * set_requested_new_state
///     * get_states
///     * get_state_change_string
///     * clear_requested_new_state
/// # Non Trivial functions to implement (even if the trivial ones are done right)
///     * external_dispatch_into_hsm: requires an understanding of how your system behaves
pub trait HsmController {
    fn init(&mut self, initial_state: StateRef) -> HSMResult<()> {
        let initial_state_id = initial_state.borrow().get_state_id();
        let states = self.get_states();
        if *initial_state_id.get_id() as usize >= states.len() {
            return Err(HSMError::InvalidStateId(format!(
                "Initial State with Id {} is not valid. There are only {} states!",
                *initial_state_id.get_id(),
                states.len() - 1
            )));
        }

        self.set_current_state(initial_state);

        Ok(())
    }

    /// Fire an event external to the HSM into it and see how it gets handled.
    /// If there is complicated threading between consumers and this HSM,
    /// override this function to navigate the ITC between them.
    // fn external_dispatch_into_hsm(&mut self, event: &dyn StateEventsIF);
    fn external_dispatch_into_hsm(
        &mut self,
        event: &dyn StateEventsIF,
    );

    fn add_state(&mut self, new_state: StateRef);
    fn get_current_state(&self) -> StateRef;
    fn set_current_state(&mut self, new_current_state: StateRef);
    fn get_states(&self) -> StatesRefVec;
    fn get_requested_new_state(&self) -> Option<StateId>;
    fn set_requested_new_state(&mut self, requested_new_state: StateId);
    fn clear_requested_new_state(&mut self);
    fn get_state_change_string(&mut self) -> &mut String;
    fn get_hsm_name(&self) -> String;

    /// Changes the state. Will IMMEDIATELY go to the LCA state, but will NOT
    /// go to target state until AFTER the handling of the current event that
    /// triggered the change_state
    fn change_state(&mut self, target_state_id: u16) {
        let target_state = StateId::new(target_state_id);
        let target_state_name = self.get_state_name(&target_state);

        assert!(
            target_state_name.is_some(),
            "change_state: Target state provided with id ({}) is invalid!",
            target_state.get_id()
        );
        debug_assert!(
            self.get_requested_new_state().is_none(),
            "change_state: A new state cannot be requested if another already is!"
        );

        let current_state = self.get_current_state();
        let current_state_id = current_state.as_ref().borrow().get_state_id().to_owned();
        let current_state_name = self.get_state_name(&current_state_id).unwrap();

        self.get_state_change_string().push_str(
            format!(
                "[change_state({} >> {} )",
                current_state_name,
                target_state_name.unwrap()
            )
            .as_str(),
        );

        if self.get_requested_new_state().is_none() {
            println!(
                "{}] - ILLEGAL NESTED STATE CHANGE IGNORED!",
                self.get_state_change_string()
            );
            return;
        }

        let target_state_ref: StateRef = self.get_state_by_id(&self.get_states(), &target_state);
        let lca_state_id = self
            .find_lca(current_state, target_state_ref)
            .expect(format!("Error finding lca for {} ", self.get_state_change_string()).as_str());

        self.exit_states_until_target(lca_state_id);

        // Alerts handle_change_state to do work at the end of handle
        self.set_requested_new_state(target_state);
    }

    /// Send an event into the HSM from within the HSM.
    /// i.e. a state fires an event while handling another event
    // fn handle_event(&mut self, event: &dyn StateEventsIF) {
    fn handle_event(
        &mut self,
        event: &dyn StateEventsIF,
    ) {
        // keep going until event is handled (true) or we reach the end
        let mut current_state = self.get_current_state();

        let hsm_name = self.get_hsm_name();

        let current_handle_string = self.get_state_change_string().clone();
        self.get_state_change_string().clear();

        self.get_state_change_string().push_str(
            format!(
                "{}: ({}) >> {}",
                hsm_name,
                current_handle_string,
                current_state.borrow().get_state_name()
            )
            .as_str(),
        );

        loop {
            let next_state = current_state.borrow().get_super_state();

            if next_state.is_none() {
                break;
            }

            let is_handled = current_state
                .borrow_mut()
                .handle_event(event);

            if is_handled {
                // event has been handled!
                break;
            }

            self.get_state_change_string()
                .push_str(format!(" > {}", current_state.borrow().get_state_name()).as_str());

            // See if parent state handles this
            current_state = next_state.unwrap();
        }
        self.handle_state_change();
    }

    fn get_state_name(&self, state_id: &StateId) -> Option<String> {
        for state_ref in &self.get_states() {
            let borrowed_state = state_ref.as_ref().borrow();
            if borrowed_state.get_state_id() == *state_id {
                return Some(borrowed_state.get_state_name().clone());
            }
        }
        None
    }

    /// Precondition - the state id is valid!
    /// todo - make this self if possible
    fn get_state_by_id(&self, states: &StatesRefVec, state_id: &StateId) -> StateRef {
        states.get(*state_id.get_id() as usize).unwrap().clone()
    }

    /// Check if a change state was enqueued while processing an event.
    /// Given that change_state exits all states up-to-but-not-including lca,
    /// enter the LCA state and then all sub-state's (including the target state).
    /// Only then do we START the target state
    fn handle_state_change(&mut self) {
        if self.get_requested_new_state().is_none() {
            println!("{}", self.get_state_change_string());
            self.get_state_change_string().clear();
            self.clear_requested_new_state();
            return;
        }

        let target_state =
            self.get_state_by_id(&self.get_states(), &self.get_requested_new_state().unwrap());
        let target_to_lca_path = target_state.borrow().get_path_to_root_state();

        self.get_state_change_string().push_str("[");

        for state_id_to_enter in target_to_lca_path.into_iter().rev() {
            let state_to_enter: StateRef =
                self.get_state_by_id(&self.get_states(), &state_id_to_enter);
            state_to_enter.as_ref().borrow_mut().handle_state_enter();

            let state_to_enter_name = state_to_enter.as_ref().borrow().get_state_name().clone();
            self.get_state_change_string()
                .push_str(format!("{}(ENTER)", state_to_enter_name).as_str());
        }

        // Start the target state!
        target_state.as_ref().borrow_mut().handle_state_start();
        let target_state_name = target_state.as_ref().borrow().get_state_name().clone();
        self.get_state_change_string()
            .push_str(format!("{}(START)]", target_state_name).as_str());

        // todo add tracing logic showing the full path!

        // Log the current chain and reset the message
        println!("{}", self.get_state_change_string());
        self.get_state_change_string().clear();
        self.clear_requested_new_state();
    }

    /// get LCA between current state and other state
    /// todo - unit test this. make this a self function
    fn find_lca(&self, source_state: StateRef, target_state: StateRef) -> HSMResult<StateId> {
        let target_state_id = target_state.as_ref().borrow().get_state_id().clone();

        if target_state_id.get_id() == &StateId::get_top_state_id() {
            return Err(HSMError::InvalidStateId(
                "There is no LCA between a state and Top!!!".to_string(),
            ));
        }

        let source_path_to_root = source_state.borrow().get_path_to_root_state();
        let target_path_to_root = target_state.borrow().get_path_to_root_state();
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
        let mut current_state = self.get_current_state();

        loop {
            let opt_parent_state = current_state.borrow().get_super_state();
            // this should only happen if we reach top
            if opt_parent_state.is_none() {
                break;
            }

            if current_state.as_ref().borrow().get_state_id() == target_state_id {
                break;
            }

            current_state.as_ref().borrow_mut().handle_state_exit();

            current_state = opt_parent_state.unwrap();
        }
    }
}
