use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventsIF,
    state::{StateChainOfResponsibility, StateId, StateRef, StatesRefVec},
};

use std::{cell::RefCell, rc::Rc};

/// The traits required to be a proper HSM controller
/// Everything is implemented for consumers.
/// The rest is implemented by HSMControllerBase.
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
    /// Fire an event external to the HSM into it and see how it gets handled.
    /// If there is complicated threading between consumers and this HSM,
    /// override this function to navigate the ITC between them.
    // fn external_dispatch_into_hsm(&mut self, event: &dyn StateEventsIF);
    fn external_dispatch_into_hsm(&mut self, event: &dyn StateEventsIF);

    fn get_current_state(&self) -> StateRef;
    fn set_current_state(&mut self, new_current_state: StateRef);
    fn get_states(&self) -> StatesRefVec;
    fn get_state_change_string(&mut self) -> &mut String;
    fn get_hsm_name(&self) -> String;

    /// Send an event into the HSM from within the HSM.
    /// i.e. a state fires an event while handling another event
    // fn handle_event(&mut self, event: &dyn StateEventsIF) {
    fn handle_event(&mut self, event: &dyn StateEventsIF) {
        // keep going until event is handled (true) or we reach the end
        let mut current_state = self.get_current_state();

        let hsm_name = self.get_hsm_name();

        self.get_state_change_string().clear();

        self.get_state_change_string().push_str(
            format!(
                "{}: {}({}): ",
                hsm_name,
                current_state.borrow().get_state_name(),
                event
            )
            .as_str(),
        );

        loop {
            let next_state = current_state.borrow().get_super_state();

            if next_state.is_none() {
                break;
            }

            let is_handled = current_state.borrow_mut().handle_event(event);

            if is_handled {
                // event has been handled!
                break;
            }

            // See if parent state handles this
            current_state = next_state.unwrap();
        }

        // Check if a state change was requested on state data cache while processing.
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
    /// Can't use indices because some consumers might not start states at 0
    fn get_state_by_id(&self, states: &StatesRefVec, state_id: &StateId) -> Option<StateRef> {
        // let state = states.get(*state_id.get_id() as usize).unwrap().clone();
        let mut found_state: Option<StateRef> = None;
        for state in states {
            if &state.borrow().get_state_id() == state_id {
                found_state = Some(state.clone());
                break;
            }
        }

        debug_assert!(
            found_state.clone().unwrap().borrow().get_state_id() == *state_id,
            "Target: {}. retrieved {}",
            state_id.get_id(),
            found_state.unwrap().borrow().get_state_id()
        );
        found_state
    }

    /// # Brief
    /// Check if a state change was requested on state data cache while processing
    /// # Details
    /// Check if a change state was enqueued while processing an event.
    /// If there is, exit all states from [current->LCA) and enter (LCA->target]
    /// THEN handle start on target.
    /// # NOTE
    /// CHANGE STATES ARE ENQUEUED BY ComposableStateData::submit_state_change_request
    fn handle_state_change(&mut self) {
        let requested_state_opt = self
            .get_current_state()
            .borrow_mut()
            .get_state_data_mut()
            .get_and_reset_requested_state_change();

        if requested_state_opt.is_none() {
            self.post_handle_event_operations();
            return;
        }

        let is_target_current = requested_state_opt.clone().unwrap().get_id()
            == self.get_current_state().borrow().get_state_id().get_id();

        // We don't clear requests once completed - requires too much mutable access
        // Just no-op on all subsequent events
        if is_target_current {
            self.post_handle_event_operations();
        }

        let requested_state = requested_state_opt.unwrap();
        let target_state_opt = self.get_state_by_id(&self.get_states(), &requested_state);

        if target_state_opt.is_none() {
            println!("Requested change state to state id {}! \
                      This is not a valid state id! Most likely your states did not start at 0 or you provided a index to high!",
                requested_state.get_id()
            );
            self.post_handle_event_operations();
            return;
        }

        let target_state = target_state_opt.unwrap();

        assert!(
            requested_state.get_id().clone() < self.get_states().len() as u16,
            "State with id {} invalid! ",
            requested_state.get_id()
        );

        let target_state_name = target_state.borrow().get_state_name();

        let current_state = self.get_current_state();

        let lca_state_id = self
            .find_lca(current_state, target_state.clone())
            .expect(format!("Error finding lca for {} ", self.get_state_change_string()).as_str());

        self.exit_states_until_target(lca_state_id);
        self.enter_states_lca_to_target(target_state.clone(), target_state_name);

        self.post_handle_event_operations();
        self.set_current_state(target_state);
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

        let mut exited_first_state = false;
        self.get_state_change_string().push_str("[");

        loop {
            let opt_parent_state = current_state.borrow().get_super_state();
            // this should only happen if we reach top
            if opt_parent_state.is_none() {
                break;
            }

            if current_state.as_ref().borrow().get_state_id() == target_state_id {
                break;
            }

            let current_state_name = current_state.borrow().get_state_name();
            if exited_first_state {
                self.get_state_change_string().push_str(", ");
            }

            self.get_state_change_string()
                .push_str(format!("{}(EXIT)", current_state_name).as_str());
            current_state.as_ref().borrow_mut().handle_state_exit();

            current_state = opt_parent_state.unwrap();
            exited_first_state = true;
        }

        self.get_state_change_string().push_str("], ");
    }

    /// Assumes we have already exited all states (non-inclusive) to the LCA
    /// Starts the target state
    fn enter_states_lca_to_target(
        &mut self,
        target_state: Rc<RefCell<dyn StateChainOfResponsibility>>,
        target_state_name: String,
    ) {
        let target_to_lca_path = target_state.borrow().get_path_to_root_state();

        // Do NOT include the LCA in the Enter's
        let mut lca_to_target_path = target_to_lca_path.into_iter().rev();
        lca_to_target_path.next();

        self.get_state_change_string().push_str("[");

        for state_id_to_enter in lca_to_target_path {
            let state_to_enter: StateRef = self
                .get_state_by_id(&self.get_states(), &state_id_to_enter)
                .unwrap();
            state_to_enter.as_ref().borrow_mut().handle_state_enter();

            let state_to_enter_name = state_to_enter.as_ref().borrow().get_state_name().clone();
            self.get_state_change_string()
                .push_str(format!("{}(ENTER), ", state_to_enter_name).as_str());
        }

        // Start the target state!
        target_state.as_ref().borrow_mut().handle_state_start();
        self.get_state_change_string()
            .push_str(format!("{}(START)]", target_state_name).as_str());
    }

    /// Operations to be performed after handling an event, regardless of outcome!
    fn post_handle_event_operations(&mut self) {
        // Log the current chain and reset the message
        println!("{}", self.get_state_change_string());
        self.get_state_change_string().clear();
    }
}
