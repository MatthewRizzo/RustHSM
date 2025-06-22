//! This file contains the logic for a state engine comprised of many
//! composable states
use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventConstraint,
    logger::HSMLogger,
    state::{StateBox, StateConstraint, StateId},
    state_engine_delegate::EngineDelegateIF,
    state_mapping::StateMapping,
    utils::{self, get_function_name, resolve_state_name},
};
use core::fmt::Display;
use log::LevelFilter;

use std::{
    cell::{Cell, RefCell},
    default::Default,
    marker::PhantomData,
    rc::Rc,
};

/// Runs the orchestration of the state 'machine' while considering its hierarchy/
/// TODO - remove RefCell for StateMapping using a builder.
// High Level: Engine owns states, states own Rc/shared reference to engine's delegate
pub struct HSMEngine<StateT: StateConstraint, EventT: StateEventConstraint> {
    pub(crate) hsm_name: String,
    pub(crate) current_state: Cell<Option<StateId>>,
    /// Used to cache the current known sequence of events and or how we handled the current event.
    pub(crate) current_handle_string: RefCell<String>,
    pub(crate) state_mapping: RefCell<StateMapping<StateT, EventT>>,
    pub(crate) logger: HSMLogger,
    // This is risky and could lead to us getting stuck!
    // These are events that are queued up while handling other events
    pending_events: RefCell<Vec<EventT>>,
    // Track if we have already changed state whole handling an event
    already_changed_state: Cell<bool>,
    /// When handling an event, it is moved/owned by us in this variable.
    /// Also acts as a tracker for if we are in the middle of handling an event.
    /// Why important? What if in handle_event, a state tells their controller to dispatch an event back at us?
    /// We use this to know that the event should be queued up.
    in_progress_event_name: RefCell<Option<String>>,
    pub(crate) phantom_state_enum: PhantomData<StateT>,
}

impl<StateT: StateConstraint, EventT: StateEventConstraint> HSMEngine<StateT, EventT> {
    /// Create an HSM engine.
    /// Highly recommend NOT exposing the HSMEngine beyond your container.
    /// Will need to be built up after the fact - via the builder!
    pub fn new(
        hsm_name: String,
        logger_level: LevelFilter,
    ) -> HSMResult<Rc<HSMEngine<StateT, EventT>>, StateT> {
        let engine = HSMEngine {
            hsm_name,
            current_state: Cell::new(None),
            current_handle_string: RefCell::new(String::new()),
            state_mapping: RefCell::new(StateMapping::<StateT, EventT>::new_default()),
            logger: HSMLogger::new(logger_level),
            pending_events: Default::default(),
            phantom_state_enum: PhantomData,
            already_changed_state: Cell::new(false),
            in_progress_event_name: RefCell::new(None),
        };
        Ok(Rc::new(engine))
    }

    pub fn get_delegate(this: &Rc<Self>) -> Rc<Self> {
        this.clone()
    }

    // Hide state ID's from users!
    /// Add the relationship between 2 states based on their id's.
    /// We have no knowledge of the state objects themselves.
    /// Helps us de-couple adding a state to the engine vs creating the states.
    pub fn add_state<T: Display + Into<u16> + From<u16>>(
        &self,
        new_state: StateBox<StateT, EventT>,
        new_state_metadata: T,
        parent_state: Option<T>,
    ) -> HSMResult<(), StateT> {
        let new_state_id = StateId::new(new_state_metadata.into());
        self.state_mapping
            .borrow_mut()
            .add_state_internal(new_state_id, parent_state)?;
        self.state_mapping
            .borrow_mut()
            .transfer_state(new_state, new_state_id)
    }

    /// Initializes the HSM - required before use!
    pub fn init(&self, starting_state: u16) -> HSMResult<(), StateT> {
        self.state_mapping.borrow().validate_cross_states()?;

        let initial_state_struct = StateId::from(starting_state);
        match self
            .state_mapping
            .borrow()
            .is_state_id_valid(&initial_state_struct)
        {
            true => Ok(()),
            false => Err(HSMError::InvalidStateId(
                StateT::from(starting_state),
                get_function_name!(),
            )),
        }?;
        self.logger.log_info(
            get_function_name!(),
            format!(
                "Initial State: {}",
                StateT::from(*initial_state_struct.get_id())
            )
            .as_str(),
        );
        self.current_state.set(Some(initial_state_struct.clone()));
        self.enter_states_lca_to_target(initial_state_struct, true)
    }

    pub fn get_current_state(&self) -> HSMResult<StateT, StateT> {
        let state: StateT = (*self
            .current_state
            .get()
            .ok_or_else(|| HSMError::EngineNotInitialized())?
            .to_owned()
            .get_id())
        .into();
        Ok(state)
    }

    /// Send an event into the HSM from within the HSM.
    /// i.e. a state fires an event while handling another event
    fn handle_event_internally(&self, event: EventT) -> HSMResult<(), StateT> {
        // keep going until event is handled (true) or we reach the end

        // State id is the variable updated each loop!
        let event_start_state_id = self
            .current_state
            .get()
            .clone()
            .ok_or_else(|| HSMError::EngineNotInitialized())?;

        // Validate the current state can handle events / is in the mapping
        match self
            .state_mapping
            .borrow()
            .is_state_valid(&event_start_state_id)
        {
            false => Err(HSMError::InvalidStateId(
                StateT::from(event_start_state_id.get_id().to_owned()),
                get_function_name!(),
            )),
            true => Ok(()),
        }?;

        let hsm_name = self.get_hsm_name();
        self.clear_handle_string();
        self.update_handle_string(
            format!(
                "{}: {}({}): ",
                hsm_name,
                resolve_state_name::<StateT>(&event_start_state_id),
                &event
            )
            .as_str(),
        );

        let mut current_state_id = event_start_state_id.to_owned();

        loop {
            let event_name = event.get_event_name().clone();
            *self.in_progress_event_name.borrow_mut() = Some(event_name.clone());
            // TODO - if the StateEventConstraint allowed an optional override to translate the args to display, this would be more useful
            // self.update_handle_string(format!("{}()", event_name).as_str());
            // self.update_handle_string("");

            let is_handled = self
                .state_mapping
                .borrow()
                .handle_event(&current_state_id, &event)?;

            if is_handled {
                break;
            }
            self.logger.log_debug(
                get_function_name!(),
                format!(
                    "{} Handling Event {}",
                    resolve_state_name::<StateT>(&current_state_id),
                    event_name,
                )
                .as_str(),
            );

            let next_state_id = match self
                .state_mapping
                .borrow()
                .get_parent_state_id(&current_state_id)
            {
                None => break, // Reached Top State
                Some(next_id) => next_id,
            };

            self.state_mapping
                .borrow()
                .is_state_id_valid_result(&next_state_id)?;

            self.logger.log_debug(
                get_function_name!(),
                format!(
                    "Letting Parent State Handle the event: {}({})",
                    event.get_event_name(),
                    utils::resolve_state_name::<StateT>(&next_state_id),
                )
                .as_str(),
            );

            // Maybe the parent state handles this
            current_state_id = next_state_id;
        }

        // If we get here, the event has been handled by at least one state (or none and we error'd)
        *self.in_progress_event_name.borrow_mut() = None;

        // Check for pending events! Doing this ensures we will always handle all pending events!
        // TODO - Is there a way we could do this asynchronously / non-recursively?
        let next_event = self.pending_events.borrow_mut().pop();
        match next_event {
            None => Ok(()),
            Some(pending_event) => self.handle_event_internally(pending_event),
        }
    }

    pub fn get_state_name(&self, state_id: &u16) -> HSMResult<String, StateT> {
        let name = StateT::from(*state_id).to_string();
        Ok(name)
    }

    /// # Brief
    /// Exit all states from [current->LCA) and enter (LCA->target]
    /// THEN handle start on target.
    /// # NOTE
    /// CHANGE STATES ARE ENQUEUED via delegate!
    fn handle_state_change(&self, requested_state: StateId) -> HSMResult<(), StateT> {
        let is_target_current = self.current_state.get() == Some(requested_state);

        // We don't clear requests once completed - requires too much mutable access
        // Just no-op on all subsequent events
        if is_target_current {
            return Ok(());
        }

        let target_state_id = {
            let target_state: StateT = self
                .state_mapping
                .borrow()
                .is_state_id_valid_result(&requested_state)
                .map(|_| StateT::from(*requested_state.get_id()))?;
            StateId::new(target_state.into())
        };

        assert!(
            self.state_mapping
                .borrow()
                .is_state_id_valid(&requested_state),
            "State with id {} invalid! ",
            requested_state.get_id()
        );

        let lca_state_id = self.find_lca(
            self.current_state
                .get()
                .ok_or_else(|| HSMError::EngineNotInitialized())?,
            requested_state,
        )?;

        if lca_state_id
            != self
                .current_state
                .get()
                .ok_or_else(|| HSMError::EngineNotInitialized())?
        {
            self.exit_states_until_target(lca_state_id)?;
        }

        self.enter_states_lca_to_target(requested_state, false)?;

        self.set_current_state(&target_state_id)?;
        self.handle_event_complete();

        Ok(())
    }

    /// get LCA between current state and other state
    fn find_lca(&self, source_state: StateId, target_state: StateId) -> HSMResult<StateId, StateT> {
        self.state_mapping
            .borrow()
            .find_lca(&source_state, &target_state)
    }

    /// Exits all states along the path to target (not including target)
    fn exit_states_until_target(&self, target_state_id: StateId) -> HSMResult<(), StateT> {
        self.update_handle_string("[");
        let mut exited_first_state = false;

        let mut current_state_id = self.current_state.get();
        match current_state_id {
            Some(_) => Ok(()),
            None => Err(HSMError::EngineNotInitialized()),
        }?;

        self.state_mapping
            .borrow()
            .is_state_id_valid_result(&current_state_id.unwrap())?;

        loop {
            match current_state_id {
                // None => break, // Happens when we reach top. The "next" computes a None
                None => break, // Happens when we reach top. The "next" computes a None
                Some(state_id) => {
                    if state_id == target_state_id {
                        // Once we reach the LCA/target, stop exiting
                        break;
                    }
                    self.state_mapping
                        .borrow()
                        .is_state_id_valid_result(&state_id)?
                }
            };
            let unwrapped_id = current_state_id.unwrap();

            let current_state_name = resolve_state_name::<StateT>(&unwrapped_id);

            if exited_first_state {
                self.update_handle_string(", ");
            }

            self.update_handle_string(format!("{}(EXIT)", current_state_name).as_str());

            // current_state_container.state_ref.handle_state_exit();
            self.state_mapping
                .borrow()
                .handle_state_exit(&unwrapped_id)?;

            let next_state_id = self.state_mapping.borrow().get_parent_state_id(
                &current_state_id.expect("Already break'd if this wasn't true!"),
            );
            current_state_id = next_state_id;
            exited_first_state = true;
        }

        self.update_handle_string("], ");
        Ok(())
    }

    /// Assumes we have already exited all states (non-inclusive) to the LCA
    /// Starts the target state
    fn enter_states_lca_to_target(
        &self,
        target_state_id: StateId,
        is_init_enter: bool,
    ) -> HSMResult<(), StateT> {
        let target_to_lca_path: Vec<StateId> = self
            .state_mapping
            .borrow()
            .resolve_path_to_root(&target_state_id)?;
        let target_state = StateT::from(*target_state_id.get_id());
        let target_state_name = target_state.to_string();

        let mut lca_to_target_path = target_to_lca_path.into_iter().rev();
        if !is_init_enter {
            // Do NOT include the LCA in the Enter's unless we are going through top at init!
            lca_to_target_path.next();
        }

        self.update_handle_string("[");

        for entering_state_id in lca_to_target_path {
            self.state_mapping
                .borrow()
                .handle_state_enter(&entering_state_id)?;

            let state_to_enter_name = resolve_state_name::<StateT>(&entering_state_id);
            self.logger.log_trace(
                get_function_name!(),
                format!("Entering {}", state_to_enter_name).as_str(),
            );
            self.update_handle_string(format!("{}(ENTER), ", state_to_enter_name).as_str());
        }

        // Start the target state!
        self.state_mapping
            .borrow()
            .handle_state_start(&target_state_id)?;
        self.logger.log_trace(
            get_function_name!(),
            format!("Starting {}", target_state_name).as_str(),
        );
        self.update_handle_string(format!("{}(START)]", target_state_name).as_str());
        Ok(())
    }

    /// Operations to be performed after handling an event, regardless of outcome!
    fn handle_event_complete(&self) {
        // Log the current chain and reset the message
        self.logger.log_info(
            get_function_name!(),
            self.current_handle_string.borrow().as_str(),
        );
        self.clear_handle_string();
    }

    /// Main API for consumers of the HSM to fire events into it.
    pub fn dispatch_event(&self, event: EventT) -> HSMResult<(), StateT> {
        let no_event_in_progress = self.in_progress_event_name.borrow().is_none();
        if no_event_in_progress {
            return self.handle_event_internally(event);
        }

        let pending_events_during_handle = !self.pending_events.borrow().is_empty();

        if pending_events_during_handle {
            // We are in the middle of handling another event and somehow a state asked their controller to handle_event
            self.pending_events.borrow_mut().push(event);
            Ok(())
        } else {
            self.handle_event_internally(event)
        }
    }

    fn set_current_state(&self, new_current_state: &StateId) -> HSMResult<(), StateT> {
        self.current_state.set(Some(*new_current_state));
        self.already_changed_state.set(false);
        Ok(())
    }

    fn update_handle_string(&self, append_str: &str) {
        self.current_handle_string.borrow_mut().push_str(append_str);
    }

    fn clear_handle_string(&self) {
        self.current_handle_string.borrow_mut().clear();
    }

    fn get_hsm_name(&self) -> String {
        self.hsm_name.clone()
    }
}

impl<StateT: StateConstraint, EventT: StateEventConstraint> EngineDelegateIF<StateT, EventT>
    for HSMEngine<StateT, EventT>
{
    fn change_state(&self, new_state: u16) -> HSMResult<(), StateT> {
        let current_event_name = match self.in_progress_event_name.borrow().as_ref() {
            None => String::from("Unknown"),
            Some(name) => name.clone(),
        };
        if self.already_changed_state.get() {
            let err = HSMError::MultipleConcurrentChangeState(
                StateT::from(new_state),
                StateT::from(
                    *self
                        .current_state
                        .get()
                        .ok_or_else(|| HSMError::EngineNotInitialized())?
                        .get_id(),
                ),
                current_event_name.to_string(),
            );
            if cfg!(test) {
                panic!("{}", err);
            } else {
                return Err(err);
            }
        }
        self.already_changed_state.set(true);
        self.handle_state_change(StateId::from(new_state))
    }

    fn internal_handle_event(&self, event: EventT) -> HSMResult<(), StateT> {
        let in_progress_event_name = match self.in_progress_event_name.borrow().clone() {
            None => "Unknown Event".to_string(),
            Some(name) => name,
        };
        self.logger.log_info(
            get_function_name!(),
            format!(
                "{}: [Adding event {} to queue while]",
                in_progress_event_name,
                event.get_event_name()
            )
            .as_str(),
        );
        self.pending_events.borrow_mut().push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn handle_state_change() {
        // todo!()
    }

    fn internal_handle_event() {
        // todo!()
    }

    #[test]
    fn handle_event_internally() {
        // todo!()
    }

    #[test]
    fn dispatch_event() {}

    #[test]
    fn find_lca() {
        // todo!()
    }

    #[test]
    fn enter_states_lca_to_target() {}

    #[test]
    fn exit_states_until_target() {}

    /// In particular, test multi-thread scenarios where concurrently:
    ///     1) External threads send events to the HSM.
    ///     2) StateT of the HSM fire events into the HSM while handling current events.
    #[test]
    fn test_many_queued_events() {}
}
