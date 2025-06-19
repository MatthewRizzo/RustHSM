use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventTrait,
    logger::HSMLogger,
    state::{StateBox, StateContainer, StateId, StateTypeTrait},
    state_engine_channel_delegate::{StateEngineDelegate, StateEngineMessages},
    state_mapping::StateMapping,
    utils::{self, get_function_name, resolve_state_name},
};
///! This file contains the logic for a state engine comprised of many
///! composable states
use core::fmt::Display;
use log::LevelFilter;

use std::{
    cell::RefCell, collections::HashMap, default::Default, fmt::format, future::Future,
    marker::PhantomData,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    RwLock,
};

/// Runs the orchestration of the state 'machine' while considering its hierarchy/
pub(crate) struct HSMEngine<StateType: StateTypeTrait, StateEvents: StateEventTrait> {
    pub(crate) hsm_name: String,
    pub(crate) current_state: StateId,
    /// Used to cache the current known sequence of events
    pub(crate) state_change_string: RefCell<String>,
    pub(crate) state_mapping: StateMapping<StateType, StateEvents>,
    pub(crate) logger: HSMLogger,
    // Rx of requests from states -> hsm. Note: currently only checked while handling other events
    // TODO - devise a method for the HSMEngine to be woken up if any requests come in
    state_proxy_requests: UnboundedReceiver<StateEngineMessages<StateType, StateEvents>>,
    // Used for internal loopback requests
    already_running: RwLock<bool>,
    pub(crate) phantom_state_enum: PhantomData<StateType>,
}

impl<StateType: StateTypeTrait, StateEvents: StateEventTrait> HSMEngine<StateType, StateEvents> {
    /// Create an HSM engine.
    /// Highly recommend NOT exposing the HSMEngine beyond your container.
    /// # Params
    pub(crate) fn new(
        hsm_name: String,
        logger: HSMLogger,
        starting_state: StateId,
        state_mapping: StateMapping<StateType, StateEvents>,
        state_proxy_requests: UnboundedReceiver<StateEngineMessages<StateType, StateEvents>>,
    ) -> HSMResult<HSMEngine<StateType, StateEvents>, StateType> {
        logger.log_info(
            get_function_name!(),
            format!(
                "Initial State: {}",
                StateType::from(*starting_state.get_id())
            )
            .as_str(),
        );

        let mut engine = HSMEngine {
            hsm_name,
            current_state: starting_state.clone(),
            state_change_string: RefCell::new(String::new()),
            state_mapping,
            logger,
            state_proxy_requests,
            already_running: RwLock::new(false),
            phantom_state_enum: PhantomData,
        };
        engine.enter_states_lca_to_target(starting_state, true)?;
        Ok(engine)
    }

    /// Send an event into the HSM from within the HSM.
    /// i.e. a state fires an event while handling another event
    fn handle_event_internally(&mut self, event: &StateEvents) -> HSMResult<(), StateType> {
        // keep going until event is handled (true) or we reach the end

        // State id is the variable updated each loop!
        let mut current_state_id = self
            .state_mapping
            .get_state_container(&self.current_state.clone())
            .ok_or_else(|| {
                HSMError::InvalidStateId(StateType::from(*self.current_state.clone().get_id()))
            })?
            .get_state_id()
            .clone();

        let hsm_name = self.get_hsm_name();
        self.clear_state_change_string();
        let evt_string = format!(
            "{}: {}({}): ",
            hsm_name,
            resolve_state_name::<StateType>(&current_state_id),
            event
        );
        self.update_state_change_string(&evt_string);
        self.logger.log_debug(get_function_name!(), &evt_string);

        loop {
            let current_state_container =
                match self.state_mapping.get_state_container(&current_state_id) {
                    None => break,
                    Some(current_state) => current_state,
                };

            let event_name = event.get_event_name().clone();
            let is_handled = current_state_container
                .state_ref
                .lock()
                .unwrap() // We only access one state at a time so it is safe
                .handle_event(&event);

            if is_handled {
                break;
            }
            self.logger.log_debug(
                get_function_name!(),
                format!(
                    "{} Handling Event {}",
                    resolve_state_name::<StateType>(&self.current_state.clone()),
                    event_name,
                )
                .as_str(),
            );

            let next_state_id = self
                .state_mapping
                .get_parent_state_id(&current_state_container.state_id);

            let next_state = match next_state_id {
                None => break,
                Some(next_id) => self
                    .state_mapping
                    .get_state_container(&next_id)
                    .ok_or_else(|| {
                        HSMError::ImpossibleStateMismatch(
                            StateType::from(*current_state_container.state_id.get_id()),
                            StateType::from(*next_id.get_id()),
                        )
                    })?,
            };

            self.logger.log_debug(
                get_function_name!(),
                format!(
                    "Letting Parent State Handle the event: {}({})",
                    event.get_event_name(),
                    utils::resolve_state_name::<StateType>(&next_state.get_state_id()),
                )
                .as_str(),
            );

            // Maybe the parent state handles this
            current_state_id = next_state.get_state_id().clone();
        }

        Ok(())
    }
    // Main entryway into the hsm. Should only be called once!
    async fn run_state_engine(&mut self) -> HSMResult<(), StateType> {
        {
            let mut rw_guard = self.already_running.write().await;
            if *rw_guard == true {
                return Err(HSMError::RunnableOnce());
            }

            *rw_guard = true;
        }

        self.logger.log_info(
            get_function_name!(),
            "Starting to run engine. Waiting for events",
        );

        // TODO - right now, this only gets woken up if an external consumer sends an event
        // It should also be woken up if anyone puts a request on the channel.
        while let Some(req) = self.state_proxy_requests.recv().await {
            match self.handle_single_proxy_request(req) {
                Ok(()) => {}
                Err(err) => {
                    self.logger.log_error(
                        "handle_single_proxy_request".to_string(),
                        &format!("Failed to process an event: {}", err),
                    );
                }
            };
        }

        Ok(())
    }

    /// Handles a single event that came off the queue/channel
    fn handle_single_proxy_request(
        &mut self,
        req: StateEngineMessages<StateType, StateEvents>,
    ) -> HSMResult<(), StateType> {
        match req {
            StateEngineMessages::ChangeState(requesting_state, new_state) => {
                let requesting_state_type = StateType::from(*requesting_state.get_id());
                let new_state_type = StateType::from(*new_state.get_id());
                self.logger.log_info(
                    get_function_name!(),
                    format!(
                        "ChangeState({} >> {}) Requested",
                        requesting_state_type, new_state_type
                    )
                    .as_str(),
                );
                self.handle_state_change(new_state.clone())?;
                assert!(self.current_state == new_state);
                self.logger
                    .log_debug(get_function_name!(), "Done handling change state!");
                Ok(())
            }
            StateEngineMessages::FireEvent(requesting_state, new_evt, on_complete_cb) => {
                let state_type = StateType::from(*requesting_state.get_id());
                self.logger.log_info(
                    get_function_name!(),
                    format!(
                        "Start - State '{}' Requests Event '{}'",
                        state_type, new_evt
                    )
                    .as_str(),
                );

                // After this point we are done with any change state from requests
                self.handle_event_internally(&new_evt)?;
                self.logger.log_info(
                    get_function_name!(),
                    format!(
                        "Done - State '{}' Requests Event '{}'",
                        state_type, &new_evt
                    )
                    .as_str(),
                );
                on_complete_cb.send(()).map_err(|_| {
                    HSMError::<StateType>::CallbackResponseNeverSentError(
                        format!("FireEvent response failed to send for event {}", new_evt)
                            .to_string(),
                    )
                })?;
                Ok(())
            }
            StateEngineMessages::GetCurrentState(response_cb) => {
                let state_id = self.current_state.clone();
                let state: StateType = state_id.get_id().clone().into();
                self.logger.log_info(
                    get_function_name!(),
                    &format!("GetCurrentState Requested. It is {}", state),
                );
                response_cb.send(state).map_err(|failed_send| {
                    HSMError::CallbackResponseNeverSentError(
                        format!("GetCurrentState response failed to send: {}", failed_send)
                            .to_string(),
                    )
                })
            }
        }
    }

    /// # Brief
    /// Exit all states from [current->LCA) and enter (LCA->target]
    /// THEN handle start on target.
    /// # NOTE
    /// CHANGE STATES ARE ENQUEUED via delegate!
    fn handle_state_change(&mut self, requested_state_change: StateId) -> HSMResult<(), StateType> {
        let is_target_current = self.current_state.clone() == requested_state_change;

        // We don't clear requests once completed - requires too much mutable access
        // Just no-op on all subsequent events
        if is_target_current {
            return Ok(());
        }

        let requested_state = requested_state_change;
        let target_state = self
            .state_mapping
            .get_state_container(&requested_state)
            .ok_or_else(|| HSMError::InvalidStateId(StateType::from(*requested_state.get_id())))
            .and_then(|container| Ok(StateType::from(*container.state_id.get_id())))?;
        let target_state_id = StateId::new(target_state.into());

        assert!(
            self.state_mapping.is_state_id_valid(&requested_state),
            "State with id {} invalid! ",
            requested_state.get_id()
        );

        let current_state_id = self.current_state.clone();

        let lca_state_id = self.find_lca(current_state_id.clone(), requested_state.clone())?;

        if lca_state_id != self.current_state {
            self.exit_states_until_target(lca_state_id)?;
        }

        self.enter_states_lca_to_target(requested_state, false)?;

        self.set_current_state(&target_state_id)?;

        self.post_handle_event_operations();

        Ok(())
    }

    /// get LCA between current state and other state
    fn find_lca(
        &self,
        source_state: StateId,
        target_state: StateId,
    ) -> HSMResult<StateId, StateType> {
        assert!(source_state != target_state);
        //  USE resolve_path_to_root from state mapping
        let source_path_to_root = self.state_mapping.resolve_path_to_root(&source_state)?;
        let target_path_to_root = self.state_mapping.resolve_path_to_root(&target_state)?;

        let mut root_to_source_path = source_path_to_root;
        root_to_source_path.reverse();
        let mut root_to_target_path = target_path_to_root;
        root_to_target_path.reverse();

        // Compare the two paths, starting from the ends of the paths (where the root is)
        // keep going until the nodes diverge. The last node before the paths diverge is the LCA.
        // let mut last_known_common_state = self.top_state_id.clone();
        // this works but is gross

        // https://stackoverflow.com/a/29504547/14810215
        // Get all differences between them...but we only care about the first
        let shared_paths = root_to_source_path
            .iter()
            .zip(root_to_target_path.iter())
            .filter(|&(source_node, target_node)| source_node.state_id == target_node.state_id)
            .collect::<Vec<(
                &&StateContainer<StateType, StateEvents>,
                &&StateContainer<StateType, StateEvents>,
            )>>();
        if shared_paths.len() == 0 {
            return Err(HSMError::LCAOfSameNode());
        }
        let last_known_common_state = shared_paths.iter().last().unwrap().0.state_id.clone();

        Ok(last_known_common_state)
    }

    /// Exits all states along the path to target (not including target)
    fn exit_states_until_target(&mut self, target_state_id: StateId) -> HSMResult<(), StateType> {
        self.update_state_change_string("[");
        let mut exited_first_state = false;

        let current_state_id = self.current_state.clone();

        let mut current_state_id = Some(
            self.state_mapping
                .get_state_container(&current_state_id)
                .ok_or_else(|| HSMError::ControllerNotInitialized())?
                .state_id
                .clone(),
        );

        loop {
            let current_state_container = match current_state_id.clone() {
                None => break, // Happens when we reach top
                Some(state_id) => self
                    .state_mapping
                    .get_state_container(&state_id)
                    .ok_or_else(|| HSMError::InvalidStateId(StateType::from(*state_id.get_id())))?,
            };

            if current_state_container.state_id == target_state_id {
                break;
            }

            let current_state_name =
                resolve_state_name::<StateType>(&current_state_id.clone().unwrap());

            if exited_first_state {
                self.update_state_change_string(", ");
            }

            self.update_state_change_string(format!("{}(EXIT)", current_state_name).as_str());

            current_state_container
                .state_ref
                .lock()
                .unwrap()
                .handle_state_exit();

            let next_state_id = self.state_mapping.get_parent_state_id(
                &current_state_id
                    .clone()
                    .expect("Already break'd if this wasn't true!"),
            );
            current_state_id = next_state_id;
            exited_first_state = true;
        }

        self.update_state_change_string("], ");
        Ok(())
    }

    /// Assumes we have already exited all states (non-inclusive) to the LCA
    /// Starts the target state
    fn enter_states_lca_to_target(
        &mut self,
        target_state_id: StateId,
        is_init_enter: bool,
    ) -> HSMResult<(), StateType> {
        let target_to_lca_path = self.state_mapping.resolve_path_to_root(&target_state_id)?;
        let target_state_container = self
            .state_mapping
            .get_state_container(&target_state_id)
            .ok_or_else(|| HSMError::InvalidStateId(StateType::from(*target_state_id.get_id())))?;
        let target_state = StateType::from(*target_state_container.state_id.get_id());
        let target_state_name = target_state.to_string();

        let mut lca_to_target_path = target_to_lca_path.into_iter().rev();
        if !is_init_enter {
            // Do NOT include the LCA in the Enter's unless we are going through top at init!
            lca_to_target_path.next();
        }

        self.update_state_change_string("[");

        for entering_state_container in lca_to_target_path {
            let state_to_enter_container = self
                .state_mapping
                .get_state_container(&entering_state_container.state_id)
                .ok_or_else(|| {
                    HSMError::InvalidStateId(StateType::from(
                        *entering_state_container.state_id.get_id(),
                    ))
                })?;
            state_to_enter_container
                .state_ref
                .lock()
                .unwrap()
                .handle_state_enter();

            let state_to_enter_name =
                resolve_state_name::<StateType>(&state_to_enter_container.state_id);
            self.logger.log_trace(
                get_function_name!(),
                format!("Entering {}", state_to_enter_name).as_str(),
            );
            self.update_state_change_string(format!("{}(ENTER), ", state_to_enter_name).as_str());
        }

        // Start the target state!
        target_state_container
            .state_ref
            .lock()
            .unwrap()
            .handle_state_start();
        self.logger.log_trace(
            get_function_name!(),
            format!("Starting {}", target_state_name).as_str(),
        );
        self.update_state_change_string(format!("{}(START)]", target_state_name).as_str());
        Ok(())
    }

    /// Operations to be performed after handling an event, regardless of outcome!
    fn post_handle_event_operations(&mut self) {
        // Log the current chain and reset the message
        // println!("{}", self.state_change_string.borrow());
        self.logger.log_info(
            "handle_state_change".to_string(),
            &format!("{}", self.state_change_string.borrow()),
        );
        self.clear_state_change_string();
    }

    fn set_current_state(&mut self, new_current_state: &StateId) -> HSMResult<(), StateType> {
        self.current_state = new_current_state.clone();
        Ok(())
    }

    fn update_state_change_string(&self, append_str: &str) {
        self.state_change_string.borrow_mut().push_str(append_str);
    }

    fn clear_state_change_string(&self) {
        self.state_change_string.borrow_mut().clear();
    }

    fn get_hsm_name(&self) -> String {
        self.hsm_name.clone()
    }
}

/// Main interface to the engine that consumers interact with once the engine is started.
/// Limits what operations it supports to just dispatching, on purpose!
/// However, it is also clone-able to have multiple accessors to the engine
pub struct HSMInterface<StateType: StateTypeTrait, StateEvents: StateEventTrait> {
    // Delegate to the full engine which we can use to wrap requests to the engine
    engine_delegate: StateEngineDelegate<StateType, StateEvents>,
}

impl<StateType: StateTypeTrait, StateEvents: StateEventTrait> HSMInterface<StateType, StateEvents> {
    pub(crate) fn new(engine_delegate: &StateEngineDelegate<StateType, StateEvents>) -> Self {
        Self {
            engine_delegate: engine_delegate.clone(),
        }
    }

    pub async fn handle_event(
        &mut self,
        event: StateEvents,
        // await_response: bool,
        // ) -> HSMResult<Option<impl Future>, StateType> {
    ) -> HSMResult<(), StateType> {
        self.engine_delegate
            .async_dispatch_event_internally(event)
            .await
    }

    pub async fn get_current_state(&self) -> HSMResult<StateType, StateType> {
        self.engine_delegate.get_current_state().await
    }
}

impl<StateType: StateTypeTrait, StateEvents: StateEventTrait> Clone
    for HSMInterface<StateType, StateEvents>
{
    fn clone(&self) -> Self {
        Self {
            engine_delegate: self.engine_delegate.clone(),
        }
    }
}

/// Struct encapsulating the process of building an HsmController.
/// Enforces immutability of the controller as states are added.
/// Effectively the public API to the controller for consumers.
/// After it is destroyed / init is called, the controller is self-managing
pub struct HSMEngineBuilder<StateType: StateTypeTrait, StateEvents: StateEventTrait> {
    unfinished_state_map: HashMap<StateId, StateContainer<StateType, StateEvents>>,
    // Resolve parent states into refs only once all states have been added!
    unfinished_state_parent_map: HashMap<StateId, StateId>,
    state_added: Vec<StateId>,
    hsm_name: String,
    top_state_id: StateId,
    logger: HSMLogger,
    engine_log_level: HSMLogger,
    delegates_provided: RefCell<Vec<StateId>>,
    delegate_log_level: log::LevelFilter,
    engine_delegate_rx: UnboundedReceiver<StateEngineMessages<StateType, StateEvents>>,
    state_delegate_tx: UnboundedSender<StateEngineMessages<StateType, StateEvents>>,
    loopback_delegate: StateEngineDelegate<StateType, StateEvents>,
}

// TODO - can we remove static??
impl<StateType: StateTypeTrait + 'static, StateEvents: StateEventTrait + 'static>
    HSMEngineBuilder<StateType, StateEvents>
{
    pub fn new(
        hsm_name: String,
        top_state_id: u16,
        builder_logger_level: LevelFilter,
        engine_log_level: LevelFilter,
        delegate_log_level: log::LevelFilter,
    ) -> HSMEngineBuilder<StateType, StateEvents> {
        let (state_delegate_tx, engine_delegate_rx) =
            unbounded_channel::<StateEngineMessages<StateType, StateEvents>>();
        // 0 is reserved / invalid because it owned by the HSM
        let loopback_delegate =
            StateEngineDelegate::new(state_delegate_tx.clone(), StateId::new(0), engine_log_level);
        HSMEngineBuilder {
            // controller_under_construction: controller,
            unfinished_state_map: Default::default(),
            unfinished_state_parent_map: Default::default(),
            state_added: Default::default(),
            hsm_name,
            top_state_id: StateId::new(top_state_id),
            logger: builder_logger_level.into(),
            engine_log_level: engine_log_level.into(),
            delegates_provided: Default::default(),
            delegate_log_level,
            engine_delegate_rx,
            state_delegate_tx,
            loopback_delegate,
        }
    }

    pub fn create_delegate(
        &self,
        requested_state_for_delegate: u16,
    ) -> HSMResult<StateEngineDelegate<StateType, StateEvents>, StateType> {
        let state = match StateType::try_from(requested_state_for_delegate) {
            Err(_) => Err(HSMError::NotAState(requested_state_for_delegate)),
            Ok(state) => Ok(state),
        }?;
        let delegate_already_created: bool = self
            .delegates_provided
            .borrow()
            .iter()
            .find(|state_already_delegated| {
                *state_already_delegated.get_id() == state.clone().into()
            })
            .is_some();
        match delegate_already_created {
            true => Err(HSMError::AlreadyDelegated(state)),
            false => {
                let state_id = StateId::new(requested_state_for_delegate);
                self.delegates_provided.borrow_mut().push(state_id.clone());
                Ok(StateEngineDelegate::new(
                    self.state_delegate_tx.clone(),
                    state_id,
                    self.delegate_log_level,
                ))
            }
        }
    }

    // Hide state ID's from users!
    pub fn add_state<T: Display + Into<u16> + From<u16>>(
        mut self,
        new_state: StateBox<StateType, StateEvents>,
        new_state_metadata: T,
        parent_state: Option<T>,
    ) -> Self {
        let new_state_id = StateId::new(new_state_metadata.into());
        let new_state_container: StateContainer<StateType, StateEvents> =
            StateContainer::new(new_state_id.clone(), new_state);
        if new_state_id != self.top_state_id && parent_state.is_none() {
            panic!("You reserved StateId {} as the Tup, but state {} does not have parents. There cannot be 2 tops states!",
                self.top_state_id,
                new_state_id
            );
        }

        // Validate the state has not been added already!

        self.unfinished_state_map
            .insert(new_state_id.clone(), new_state_container);

        let mut parent_state_id: Option<StateId> = None;
        if parent_state.is_some() {
            let parent_state_metadata = parent_state.unwrap();
            parent_state_id = Some(StateId::new(parent_state_metadata.into()));
            self.unfinished_state_parent_map
                .insert(new_state_id.clone(), parent_state_id.clone().unwrap());
        }
        self.state_added.push(new_state_id.clone());

        self.logger.log_debug(
            get_function_name!(),
            format!(
                "{}) Adding state {} with parent {}",
                LevelFilter::Info.as_str(),
                resolve_state_name::<StateType>(&new_state_id),
                match parent_state_id {
                    None => "None".to_owned(),
                    Some(parent_id) => resolve_state_name::<StateType>(&parent_id),
                },
            )
            .as_str(),
        );

        self
    }

    /// Final step in process
    pub fn init(
        self,
        initial_state_id: u16,
    ) -> HSMResult<HSMInterface<StateType, StateEvents>, StateType> {
        let initial_state_id_struct = StateId::new(initial_state_id);

        let state_mapping = StateMapping::new(
            self.unfinished_state_map,
            self.unfinished_state_parent_map,
            Some(self.logger),
        );
        match state_mapping.is_state_id_valid(&initial_state_id_struct) {
            true => Ok(()),
            false => Err(HSMError::InvalidStateId(StateType::from(initial_state_id))),
        }?;
        assert!(state_mapping.validate_cross_states());

        let interface = HSMInterface::new(&self.loopback_delegate);
        let mut engine = HSMEngine::new(
            self.hsm_name,
            self.engine_log_level,
            initial_state_id.into(),
            state_mapping,
            self.engine_delegate_rx,
        )?;

        let handle = tokio::spawn(async move { engine.run_state_engine().await });

        Ok(interface)
    }
}

#[cfg(test)]
mod tests {
    use crate::examples::*;
    use crate::test_utils::*;

    #[test]
    fn test_handle_state_change() {
        let _ = build_test_hsm(ExampleStates::Top);
    }

    #[test]
    fn test_handle_single_proxy_request() {
        let _ = build_test_hsm(ExampleStates::Top);
    }

    #[test]
    fn test_process_proxy_requests() {}

    #[test]
    fn test_handle_event_internally() {
        // todo!()
    }

    #[test]
    fn test_dispatch_event() {}

    #[test]
    fn test_find_lca() {
        // todo!()
    }

    #[test]
    fn test_enter_states_lca_to_target() {}

    #[test]
    fn test_exit_states_until_target() {}

    /// In particular, test multi-thread scenarios where concurrently:
    ///     1) External threads send events to the HSM.
    ///     2) States of the HSM fire events into the HSM while handling current events.
    #[test]
    fn test_many_queued_events() {}
}
