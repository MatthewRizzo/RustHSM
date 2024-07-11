///! Module encapsulating how states are built
use crate::{
    errors::{HSMError, HSMResult},
    state::StateRef,
    state_data_delegate::{StateDataDelegate, StateDelegateRef},
};

/// Breaks build-time circular dependency of States and their delegate.
/// Meant to be used by consumers as the build their states!
pub struct StateBuilder {
    delegate_under_construction: StateDelegateRef,
    state_under_construction: Option<StateRef>,
    state_id: u16,
    state_name: String,
    parent_delegate: Option<StateDelegateRef>,
}

impl StateBuilder {
    /// Breaks the deadlock for the delegate<->state relationship by introducing
    /// a builder that temporary delegate and None for the current state.
    /// It will provide you with the delegate to inject into your stateIF child.
    /// With a created stateIF, inject it back into the builder.
    /// The builder will manage swapping out the temporary delegate(s) with
    /// real data now that all data is known.
    /// Unbeknownst to you as a consumer, your state will not crash if used.
    pub fn new(
        state_id: u16,
        state_name: String,
        parent_delegate: Option<StateDelegateRef>,
    ) -> Self {
        StateBuilder {
            delegate_under_construction: StateDataDelegate::build_temporary(),
            state_under_construction: None,
            state_id,
            state_name,
            parent_delegate,
        }
    }

    /// Use this API to finish building your state, provide us back your state
    /// state constructor so all steps can be in 1 chain.
    pub fn get_delegate(&self) -> StateDelegateRef {
        self.delegate_under_construction.clone()
    }

    /// Now that the builder provides a delegate, create your true state with it
    /// and then provide us the real state
    pub fn set_concrete_state(mut self, real_state: StateRef) -> Self {
        let real_delegate = StateDataDelegate::build_real(
            self.state_id,
            self.state_name.clone(),
            real_state.clone(),
            self.parent_delegate.clone(),
        );

        real_state
            .borrow_mut().get_state_data()
            .borrow_mut().set_details(real_delegate.clone())
            .expect("Builder failed to set_details. Did you order the building of your state correctly?");

        self.delegate_under_construction = real_delegate;
        self.state_under_construction = Some(real_state);
        self
    }

    /// Validates that the state and its pieces have all been built correctly!
    pub fn validate_build(&self) -> HSMResult<()> {
        if self.state_under_construction.is_none() {
            return Err(HSMError::ConcreteStateNotProvidedToBuilder());
        }

        self.delegate_under_construction
            .clone()
            .borrow()
            .get_details()?;
        Ok(())
    }
}
