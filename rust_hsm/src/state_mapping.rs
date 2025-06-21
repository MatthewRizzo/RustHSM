use log::{self, LevelFilter};
///! This file contains the logic for how states are grouped together.
///! Using this info, the entire "tree" of states can be resolved!
use std::{cell::RefCell, collections::HashMap};

use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventConstraint,
    logger::HSMLogger,
    state::{StateBox, StateConstraint, StateContainer, StateId},
    utils::{get_function_name, resolve_state_name},
};

/// # Brief
/// It protects / hides the dyn states from us.
/// In exchange, it allows us access API's when we provide tokens (StateId).
/// Similarly, when it reports info back to us, it does so with tokens.
pub(crate) struct StateMapping<StateT: StateConstraint, EventT: StateEventConstraint> {
    top_state_id: RefCell<Option<StateId>>,
    // state id -> state
    state_map: HashMap<StateId, StateContainer<StateT, EventT>>,
    /// stateid -> parent state
    /// If the node has a parent, it is in the map!
    /// If it is not present....it is an orphan (Top)
    state_parent_map: HashMap<StateId, StateId>,
    logger: HSMLogger,
}

impl<StateT: StateConstraint, EventT: StateEventConstraint> StateMapping<StateT, EventT> {
    pub(crate) fn new(
        top_state_id: StateId,
        state_map: HashMap<StateId, StateContainer<StateT, EventT>>,
        raw_state_parent_map: HashMap<StateId, StateId>,
        logger: Option<HSMLogger>,
    ) -> Self {
        state_map.keys().len();

        Self {
            top_state_id: RefCell::new(Some(top_state_id)),
            state_map,
            state_parent_map: raw_state_parent_map,
            logger: logger.unwrap_or(HSMLogger::from(LevelFilter::Info)),
        }
    }

    pub(crate) fn new_default() -> Self {
        Self {
            top_state_id: RefCell::new(None),
            state_map: HashMap::new(),
            state_parent_map: HashMap::new(),
            logger: HSMLogger::from(LevelFilter::Info),
        }
    }

    pub(crate) fn get_registered_state_id(&self) -> Vec<StateId> {
        self.state_parent_map.keys().cloned().collect()
    }

    pub(crate) fn transfer_state(
        &mut self,
        new_state: StateBox<StateT, EventT>,
        new_state_id: StateId,
    ) -> HSMResult<(), StateT> {
        let new_state_container: StateContainer<StateT, EventT> =
            StateContainer::new(new_state_id.clone(), new_state);

        // Validate the state has not been added already!
        match self
            .state_map
            .insert(new_state_id.clone(), new_state_container)
        {
            None => Ok(()),
            Some(_) => Err(HSMError::AddDuplicateStateId(
                StateT::from(new_state_id.get_id().clone()),
                new_state_id.clone(),
            )),
        }?;

        Ok(())
    }

    pub(crate) fn add_state_internal<T: std::fmt::Display + Into<u16> + From<u16>>(
        &mut self,
        new_state_id: StateId,
        parent_state: Option<T>,
    ) -> HSMResult<(), StateT> {
        let new_state_name = resolve_state_name::<StateT>(&new_state_id);
        if let Some(chosen_top) = self.top_state_id.borrow().clone() {
            if new_state_id != chosen_top && parent_state.is_none() {
                return Err(HSMError::MultipleTopState(
                    resolve_state_name::<StateT>(&chosen_top),
                    chosen_top.clone(),
                    new_state_name,
                    new_state_id,
                ));
            }
        };

        let mut parent_state_id: Option<StateId> = None;
        if parent_state.is_some() {
            let parent_state_metadata = parent_state.unwrap();
            parent_state_id = Some(StateId::new(parent_state_metadata.into()));
            self.state_parent_map
                .insert(new_state_id.clone(), parent_state_id.clone().unwrap());
        }

        self.logger.log_debug(
            get_function_name!(),
            format!(
                "{}) Adding state {} with parent {}",
                LevelFilter::Info.as_str(),
                new_state_name,
                match parent_state_id {
                    None => "None".to_owned(),
                    Some(parent_id) => resolve_state_name::<StateT>(&parent_id),
                },
            )
            .as_str(),
        );
        Ok(())
    }

    /// Return the id of state's parent
    pub(crate) fn get_parent_state_id(&self, id: &StateId) -> Option<StateId> {
        match self.state_parent_map.get(id) {
            None => None,
            Some(parent) => Some(parent.clone()),
        }
    }

    pub(crate) fn is_state_valid(&self, id: &StateId) -> bool {
        match self.state_map.get(id) {
            None => false,
            Some(_) => true,
        }
    }

    pub(crate) fn handle_event(&self, id: &StateId, event: &EventT) -> HSMResult<bool, StateT> {
        match self.state_map.get(id) {
            None => Err(HSMError::InvalidStateId(
                StateT::from(*id.get_id()),
                get_function_name!(),
            )),
            Some(container) => Ok(container.state_ref.handle_event(&event)),
        }
    }

    pub(crate) fn handle_state_enter(&self, id: &StateId) -> HSMResult<(), StateT> {
        match self.state_map.get(id) {
            None => Err(HSMError::InvalidStateId(
                StateT::from(*id.get_id()),
                get_function_name!(),
            )),
            Some(container) => Ok(container.state_ref.handle_state_enter()),
        }
    }

    pub(crate) fn handle_state_start(&self, id: &StateId) -> HSMResult<(), StateT> {
        match self.state_map.get(id) {
            None => Err(HSMError::InvalidStateId(
                StateT::from(*id.get_id()),
                get_function_name!(),
            )),
            Some(container) => Ok(container.state_ref.handle_state_start()),
        }
    }

    pub(crate) fn handle_state_exit(&self, id: &StateId) -> HSMResult<(), StateT> {
        match self.state_map.get(id) {
            None => Err(HSMError::InvalidStateId(
                StateT::from(*id.get_id()),
                get_function_name!(),
            )),
            Some(container) => Ok(container.state_ref.handle_state_exit()),
        }
    }

    /// get LCA between current state and other state
    pub(crate) fn find_lca(
        &self,
        source_state: &StateId,
        target_state: &StateId,
    ) -> HSMResult<StateId, StateT> {
        assert!(source_state != target_state);
        //  USE resolve_path_to_root from state mapping
        let source_path_to_root = self.resolve_path_to_root(source_state)?;
        let target_path_to_root = self.resolve_path_to_root(target_state)?;

        let mut root_to_source_path = source_path_to_root;
        root_to_source_path.reverse();
        let mut root_to_target_path = target_path_to_root;
        root_to_target_path.reverse();

        // Compare the two paths, starting from the ends of the paths (where the root is)
        // keep going until the nodes diverge. The last node before the paths diverge is the LCA.
        // this works but is gross

        // https://stackoverflow.com/a/29504547/14810215
        // Get all differences between them...but we only care about the first
        let shared_paths = root_to_source_path
            .iter()
            .zip(root_to_target_path.iter())
            .filter(|&(source_node, target_node)| source_node == target_node)
            .collect::<Vec<(&StateId, &StateId)>>();
        if shared_paths.len() == 0 {
            return Err(HSMError::LCAOfSameNode());
        }
        let last_known_common_state = shared_paths.iter().last().unwrap().0.clone();

        Ok(last_known_common_state)
    }

    /// Path to root node from provided node (inclusive on both ends!)
    /// [start, ..., end]
    pub(crate) fn resolve_path_to_root(
        &self,
        start_node: &StateId,
    ) -> HSMResult<Vec<StateId>, StateT> {
        let mut current_node_id = start_node.clone();
        let current_node = self
            .state_map
            .get(&start_node)
            .ok_or_else(|| {
                HSMError::InvalidStateId(StateT::from(*start_node.get_id()), get_function_name!())
            })?
            .state_id
            .clone();
        let mut path_to_root: Vec<StateId> = vec![current_node];

        loop {
            self.logger.log_debug(
                get_function_name!(),
                format!("Current State: {}", StateT::from(*current_node_id.get_id())).as_str(),
            );
            current_node_id = match self.get_parent_state_id(&current_node_id) {
                None => {
                    self.logger
                        .log_debug(get_function_name!(), "Next State: None. Stopping");
                    break;
                }
                Some(parent_id) => {
                    match self.is_state_id_valid(&parent_id) {
                        false => Err(HSMError::ImpossibleStateMismatch(
                            StateT::from(*current_node_id.get_id()),
                            StateT::from(*parent_id.get_id()),
                        )),
                        true => Ok(()),
                    }?;
                    path_to_root.push(parent_id.clone());
                    self.logger.log_debug(
                        get_function_name!(),
                        format!(
                            "Next State: {}",
                            StateT::from(*parent_id.get_id()).to_string()
                        )
                        .as_str(),
                    );
                    parent_id
                }
            };
        }

        Ok(path_to_root)
    }

    // Given 2 mappings of containers, validate no illegal combinations exist
    // Pan
    pub(crate) fn validate_cross_states(&self) -> HSMResult<(), StateT> {
        for (parent_lookup_id, parent_id) in &self.state_parent_map {
            if self.state_map.get(&parent_lookup_id).is_none() {
                let msg = format!(
                    "State id from parent map {} does not exist in state map!",
                    parent_lookup_id
                );
                self.logger.log_error(get_function_name!(), msg.as_str());
                return Err(HSMError::MapValidationError(msg));
            }

            if self.state_map.get(&parent_id).is_none() {
                let msg = format!(
                    "Parent State id from parent map {} does not exist in state map!",
                    parent_id
                );
                self.logger.log_error(get_function_name!(), msg.as_str());
                return Err(HSMError::MapValidationError(msg));
            }
        }

        // Do we need to check if both have the same size?
        Ok(())
    }

    pub(crate) fn is_state_id_valid(&self, state_id: &StateId) -> bool {
        self.state_map.contains_key(state_id)
    }
    pub(crate) fn is_state_id_valid_result(&self, state_id: &StateId) -> HSMResult<(), StateT> {
        match self.is_state_id_valid(state_id) {
            true => Ok(()),
            false => Err(HSMError::InvalidStateId(
                StateT::from(*state_id.get_id()),
                get_function_name!(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use crate::examples::*;
    use crate::test_utils::*;
    use log::LevelFilter;

    use super::*;

    // fn do_paths_match(a: &Vec<StateContainer<ExampleStates>>, b: &Vec<StateContainer<ExampleStates>>) -> bool {
    fn do_paths_match(a: &Vec<StateId>, b: &Vec<StateId>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }

    fn resolve_path_to_id(path: &Vec<StateId>) -> Vec<StateId> {
        path.iter().map(|state_id| state_id.clone()).collect()
    }

    #[test]
    fn parent_link() {
        let test_logger = HSMLogger::new(LevelFilter::Trace);
        let mut state_map = HashMap::<StateId, StateContainer<ExampleStates, ExampleEvents>>::new();
        let mut raw_parent_map = HashMap::<StateId, StateId>::new();
        let mut num_states_created: u16 = 0;

        let top_state = DummyStateStruct::new(&mut num_states_created);
        assert!(num_states_created == 1);
        let a1_state = DummyStateStruct::new(&mut num_states_created);
        assert!(num_states_created == 2);
        let b1_state = DummyStateStruct::new(&mut num_states_created);
        assert!(num_states_created == 3);
        let a2_state = DummyStateStruct::new(&mut num_states_created);
        assert!(num_states_created == 4);
        let top_container = StateContainer::new(StateId::new(ExampleStates::Top.into()), top_state);
        let a1_container =
            StateContainer::new(StateId::new(ExampleStates::LevelA1.into()), a1_state);
        let b1_container =
            StateContainer::new(StateId::new(ExampleStates::LevelB1.into()), b1_state);
        let a2_container =
            StateContainer::new(StateId::new(ExampleStates::LevelA2.into()), a2_state);
        state_map.insert(ExampleStates::Top.into(), top_container);
        state_map.insert(ExampleStates::LevelA1.into(), a1_container);
        state_map.insert(ExampleStates::LevelB1.into(), b1_container);
        state_map.insert(ExampleStates::LevelA2.into(), a2_container);

        raw_parent_map.insert(ExampleStates::LevelA1.into(), ExampleStates::Top.into());
        raw_parent_map.insert(ExampleStates::LevelB1.into(), ExampleStates::Top.into());
        raw_parent_map.insert(ExampleStates::LevelA2.into(), ExampleStates::LevelA1.into());

        let mapping = StateMapping::<ExampleStates, ExampleEvents>::new(
            ExampleStates::Top.into(),
            state_map,
            raw_parent_map,
            Some(LevelFilter::Trace.into()),
        );
        test_logger.log_info(get_function_name!(), "Assembled the mappings!");

        assert!(mapping
            .get_parent_state_id(&ExampleStates::Top.into())
            .is_none());
        assert_eq!(
            mapping
                .get_parent_state_id(&ExampleStates::LevelA1.into())
                .unwrap(),
            ExampleStates::Top.into()
        );
        assert_eq!(
            mapping
                .get_parent_state_id(&ExampleStates::LevelB1.into())
                .unwrap(),
            ExampleStates::Top.into()
        );
        assert_eq!(
            mapping
                .get_parent_state_id(&ExampleStates::LevelA2.into())
                .unwrap(),
            ExampleStates::LevelA1.into()
        );

        assert!(!mapping.is_state_id_valid(&StateId::from(7)));
        assert!(mapping.is_state_id_valid(&ExampleStates::Top.into()));
        assert!(mapping.is_state_id_valid(&ExampleStates::LevelA1.into()));
        assert!(mapping.is_state_id_valid(&ExampleStates::LevelB1.into()));
        assert!(mapping.is_state_id_valid(&ExampleStates::LevelA2.into()));
        assert!(!mapping.is_state_id_valid(&ExampleStates::INVALID.into()));

        assert!(mapping.validate_cross_states().is_ok());
        test_logger.log_info(
            get_function_name!(),
            "Passed verifying raw data structures! Onto Algorithms",
        );

        {
            let path = mapping.resolve_path_to_root(&ExampleStates::Top.into());
            assert!(path.is_ok());
            let id_paths = resolve_path_to_id(&path.unwrap());
            let expected_id_path: Vec<StateId> = vec![ExampleStates::Top.into()];
            assert!(
                do_paths_match(&id_paths, &expected_id_path),
                "expected {:?}. Received {:?} ",
                expected_id_path,
                id_paths
            );
        }
        {
            let path = mapping.resolve_path_to_root(&ExampleStates::LevelA1.into());
            assert!(path.is_ok());
            let id_paths = resolve_path_to_id(&path.unwrap());
            let expected_id_path: Vec<StateId> =
                vec![ExampleStates::LevelA1.into(), ExampleStates::Top.into()];
            assert!(
                do_paths_match(&id_paths, &expected_id_path),
                "expected {:?}. Received {:?} ",
                cast_id_vector(&expected_id_path),
                cast_id_vector(&id_paths)
            );
            test_logger.log_info(get_function_name!(), "A1 -> Root is good");
        }
        {
            let path = mapping.resolve_path_to_root(&ExampleStates::LevelB1.into());
            assert!(path.is_ok());
            let id_paths = resolve_path_to_id(&path.unwrap());
            let expected_id_path: Vec<StateId> =
                vec![ExampleStates::LevelB1.into(), ExampleStates::Top.into()];
            assert!(
                do_paths_match(&id_paths, &expected_id_path),
                "expected {:?}. Received {:?} ",
                cast_id_vector(&expected_id_path),
                cast_id_vector(&id_paths)
            );
            test_logger.log_info(get_function_name!(), "B1 -> Root is good");
        }
        {
            let path = mapping.resolve_path_to_root(&ExampleStates::LevelA2.into());
            assert!(path.is_ok());
            let id_paths = resolve_path_to_id(&path.unwrap());
            let expected_id_path: Vec<StateId> = vec![
                ExampleStates::LevelA2.into(),
                ExampleStates::LevelA1.into(),
                ExampleStates::Top.into(),
            ];
            assert!(
                do_paths_match(&id_paths, &expected_id_path),
                "expected {:?}. Received {:?} ",
                cast_id_vector(&expected_id_path),
                cast_id_vector(&id_paths)
            );
            test_logger.log_info(get_function_name!(), "A2 -> Root is good");
        }
        {
            let path = mapping.resolve_path_to_root(&ExampleStates::INVALID.into());
            assert!(path.is_err());
        }
    }
}
