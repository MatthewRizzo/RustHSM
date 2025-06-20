use log::{self, LevelFilter};
///! This file contains the logic for how states are grouped together.
///! Using this info, the entire "tree" of states can be resolved!
use std::collections::HashMap;

use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventTrait,
    logger::HSMLogger,
    state::{StateContainer, StateId, StateTypeTrait},
    utils::get_function_name,
};

pub(crate) struct StateMapping<StateType: StateTypeTrait, StateEvents: StateEventTrait> {
    // state id -> state
    state_map: HashMap<StateId, StateContainer<StateType, StateEvents>>,
    /// stateid -> parent state
    /// If the node has a parent, it is in the map!
    /// If it is not present....it is an orphan (Top)
    state_parent_map: HashMap<StateId, StateId>,
    logger: HSMLogger,
}

impl<StateType: StateTypeTrait, StateEvents: StateEventTrait> StateMapping<StateType, StateEvents> {
    pub(crate) fn new(
        state_map: HashMap<StateId, StateContainer<StateType, StateEvents>>,
        raw_state_parent_map: HashMap<StateId, StateId>,
        logger: Option<HSMLogger>,
    ) -> Self {
        state_map.keys().len();

        Self {
            state_map,
            state_parent_map: raw_state_parent_map,
            logger: logger.unwrap_or(HSMLogger::from(LevelFilter::Info)),
        }
    }

    pub(crate) fn get_state_container(
        &self,
        id: &StateId,
    ) -> Option<&StateContainer<StateType, StateEvents>> {
        match self.state_map.get(&id) {
            None => None,
            Some(state) => Some(state),
        }
    }

    /// Return the id of state's parent
    pub(crate) fn get_parent_state_id(&self, id: &StateId) -> Option<StateId> {
        match self.state_parent_map.get(id) {
            None => None,
            Some(parent) => Some(parent.clone()),
        }
    }

    /// Path to root node from provided node (inclusive on both ends!)
    /// [start, ..., end]
    pub(crate) fn resolve_path_to_root(
        &self,
        start_node: &StateId,
    ) -> HSMResult<Vec<&StateContainer<StateType, StateEvents>>, StateType> {
        let mut current_node_id = start_node.clone();
        let current_node = self
            .state_map
            .get(&start_node)
            .ok_or_else(|| HSMError::InvalidStateId(StateType::from(*start_node.get_id())))?;
        let mut path_to_root: Vec<&StateContainer<StateType, StateEvents>> = vec![current_node];

        loop {
            self.logger.log_debug(
                get_function_name!(),
                format!(
                    "Current State: {}",
                    StateType::from(*current_node_id.get_id())
                )
                .as_str(),
            );
            current_node_id = match self.get_parent_state_id(&current_node_id) {
                None => {
                    self.logger
                        .log_debug(get_function_name!(), "Next State: None. Stopping");
                    break;
                }
                Some(parent_id) => {
                    let parent_node = self.get_state_container(&parent_id).ok_or_else(|| {
                        HSMError::ImpossibleStateMismatch(
                            StateType::from(*current_node_id.get_id()),
                            StateType::from(*parent_id.get_id()),
                        )
                    })?;
                    path_to_root.push(parent_node);
                    self.logger.log_debug(
                        get_function_name!(),
                        format!(
                            "Next State: {}",
                            StateType::from(*parent_id.get_id()).to_string()
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
    pub(crate) fn validate_cross_states(&self) -> bool {
        for (parent_lookup_id, parent_id) in &self.state_parent_map {
            if self.state_map.get(&parent_lookup_id).is_none() {
                self.logger.log_error(
                    get_function_name!(),
                    format!(
                        "State id from parent map {} does not exist in state map!",
                        parent_lookup_id
                    )
                    .as_str(),
                );

                return false;
            }

            if self.state_map.get(&parent_id).is_none() {
                println!(
                    "Parent State id from parent map {} does not exist in state map!",
                    parent_id
                );
                return false;
            }
        }

        // Do we need to check if both have the same size?

        true
    }

    pub(crate) fn is_state_id_valid(&self, state_id: &StateId) -> bool {
        self.state_map.contains_key(state_id)
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

    fn resolve_path_to_id(
        path: &Vec<&StateContainer<ExampleStates, ExampleEvents>>,
    ) -> Vec<StateId> {
        path.iter()
            .map(|container| container.state_id.clone())
            .collect()
    }

    #[test]
    fn parent_link() {
        let test_logger = HSMLogger::new(LevelFilter::Trace);
        let mut state_map = HashMap::<StateId, StateContainer<ExampleStates, ExampleEvents>>::new();
        let mut raw_parent_map = HashMap::<StateId, StateId>::new();
        let mut num_states_created: u16 = 0;
        let (tx, _) = channel();

        let top_container =
            fill_state_container(ExampleStates::Top, &mut num_states_created, tx.clone());
        let a1_container =
            fill_state_container(ExampleStates::LevelA1, &mut num_states_created, tx.clone());
        let b1_container =
            fill_state_container(ExampleStates::LevelB1, &mut num_states_created, tx.clone());
        let a2_container =
            fill_state_container(ExampleStates::LevelA2, &mut num_states_created, tx);
        state_map.insert(ExampleStates::Top.into(), top_container);
        state_map.insert(ExampleStates::LevelA1.into(), a1_container);
        state_map.insert(ExampleStates::LevelB1.into(), b1_container);
        state_map.insert(ExampleStates::LevelA2.into(), a2_container);

        raw_parent_map.insert(ExampleStates::LevelA1.into(), ExampleStates::Top.into());
        raw_parent_map.insert(ExampleStates::LevelB1.into(), ExampleStates::Top.into());
        raw_parent_map.insert(ExampleStates::LevelA2.into(), ExampleStates::LevelA1.into());

        let mapping = StateMapping::<ExampleStates, ExampleEvents>::new(
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

        assert!(mapping.validate_cross_states());
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
