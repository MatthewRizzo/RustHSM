use rust_hsm::{self, state::StateId};
mod light_hsm;
use light_hsm::{
    light_events, light_events::LightEvents, light_hsm_controller::LightControllerHsm,
    light_hsm_data, light_states, light_states::LightStates,
};

fn main() {
    let mut light_hsm = LightControllerHsm::new();

    let starting_state = light_hsm.get_current_state();
    assert!(starting_state.borrow().get_state_id().get_id().clone() == LightStates::DIMMER as u16);

    // Set the dimmer value that triggers another internal event being fired for OFF
    {
        light_hsm.dispatch_into_hsm(LightEvents::Set(0));
        let state = light_hsm.get_current_state();
        let state_id = state.borrow().get_state_id();
        let expected_state_id = StateId::new(LightStates::OFF as u16);
        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id,
            state.borrow().get_state_id().get_id()
        );

        let data = light_hsm.get_light_data();
        assert_eq!(data.borrow().light_percentage, 0);
    }
    // Cause a no-op, we are already on! - Test an un-handled event
    {
        light_hsm.dispatch_into_hsm(LightEvents::TurnOn);

        let state = light_hsm.get_current_state();
        let state_id = state.borrow().get_state_id();
        let expected_state_id = StateId::new(LightStates::ON as u16);

        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id,
            state.borrow().get_state_id().get_id()
        );
    }
    // Cause a state change via turn off event (by levering parent behavior!)
    {
        light_hsm.dispatch_into_hsm(LightEvents::TurnOff);

        let state = light_hsm.get_current_state();
        let state_id = state.borrow().get_state_id();
        let expected_state_id = StateId::new(LightStates::OFF as u16);
        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id,
            state.borrow().get_state_id().get_id()
        );
    }
}
