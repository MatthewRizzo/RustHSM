use rust_hsm::{self, state::StateId};
mod light_hsm;
use light_hsm::{
    light_events, light_events::LightEvents, light_hsm_controller::LightControllerHsm,
    light_hsm_data, light_states, light_states::LightStates,
};

fn main() {
    let light_hsm = LightControllerHsm::new();

    let starting_state = light_hsm.borrow().get_current_state();
    assert!(starting_state.borrow().get_state_id().get_id().clone() == LightStates::OFF as u16);

    {
        light_hsm.borrow().dispatch_into_hsm(LightEvents::TurnOn);

        let state = light_hsm.borrow().get_current_state();
        let state_id = state.borrow().get_state_id();
        let expected_state_id = StateId::new(LightStates::ON as u16);

        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id,
            state.borrow().get_state_id().get_id()
        );
    }
    // Test an un-handled event
    {
        light_hsm.borrow().dispatch_into_hsm(LightEvents::TurnOn);

        let state = light_hsm.borrow().get_current_state();
        let state_id = state.borrow().get_state_id();
        let expected_state_id = StateId::new(LightStates::ON as u16);
        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id,
            state.borrow().get_state_id().get_id()
        );
    }
}
