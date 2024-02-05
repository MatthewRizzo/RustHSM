use crate::callback::Callback;
use std::{collections::HashMap, fmt::Display, hash::Hash};

pub struct CallbackRegistryPair<Input, Key, Output>(Key, Callback<Input, Output>)
where
    Key: Eq + PartialEq + Hash;

pub struct CallbackRegistry<Input, Key, Output>
where
    Key: Eq + PartialEq + Hash + Display,
{
    callbacks: HashMap<Key, Callback<Input, Output>>,
}

impl<Input, Key, Output> CallbackRegistry<Input, Key, Output>
where
    Key: Eq + PartialEq + Hash + Display,
{
    /// RAII register all function handlers!
    /// If multiple handlers for the same function are registered, first one wins!
    pub fn new(handlers: Vec<CallbackRegistryPair<Input, Key, Output>>) -> Self {
        let mut callbacks: HashMap<Key, Callback<Input, Output>> = HashMap::new();

        for handler_pair in handlers.into_iter() {
            if callbacks.contains_key(&handler_pair.0) {
                continue;
            }

            callbacks.insert(handler_pair.0, handler_pair.1);
        }

        Self { callbacks }
    }

    /// # Return
    /// * None if key is not present
    /// * Output if key is present. Also executes the callback!
    pub fn dispatch_to_registry(&self, key: &Key, args: Input) -> Option<Output> {
        match self.callbacks.contains_key(key) {
            true => {
                // Don't consume the registered callback, just this copy of it
                let cb = self.callbacks.get(key).clone();
                cb.unwrap().fire(args)
            }
            false => None,
        }
    }
}
