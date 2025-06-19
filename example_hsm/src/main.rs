mod light_hsm;
use light_hsm::{
    light_events, light_events::LightEvents, light_hsm_controller::LightControllerHsm,
    light_hsm_data, light_states, light_states::LightStates,
};

fn main() {
    let mut light_hsm = LightControllerHsm::new(log::LevelFilter::Info);

    let starting_state = light_hsm.get_current_state();
    assert!(
        starting_state == LightStates::DIMMER,
        "starting_state = {}. Expected: {}",
        starting_state,
        LightStates::DIMMER
    );

    // Check setup / init. Top -> On -> DIMMER
    {
        let data = light_hsm.get_light_data();
        assert_eq!(data.borrow().top_enter_called, 1);
        assert_eq!(data.borrow().top_start_called, 0);
        assert_eq!(data.borrow().top_exit_called, 0);
        assert_eq!(data.borrow().on_enter_called, 1);
        assert_eq!(data.borrow().on_start_called, 0);
        assert_eq!(data.borrow().on_exit_called, 0);
        assert_eq!(data.borrow().dimmer_enter_called, 1);
        assert_eq!(data.borrow().dimmer_start_called, 1);
        assert_eq!(data.borrow().dimmer_exit_called, 0);
        assert_eq!(data.borrow().off_enter_called, 0);
        assert_eq!(data.borrow().off_start_called, 0);
        assert_eq!(data.borrow().off_exit_called, 0);
        data.borrow_mut().clear_counts();
    }

    // Set the dimmer value that triggers another internal event being fired for OFF
    {
        light_hsm
            .dispatch_into_hsm(LightEvents::Set(0))
            .expect("Error dispatching Set event into HSM");

        let state_id = light_hsm.get_current_state();
        let expected_state_id = LightStates::OFF;
        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id.as_ref(),
            state_id.as_ref()
        );

        let data = light_hsm.get_light_data();
        assert_eq!(data.borrow().light_percentage, 0);
        assert_eq!(data.borrow().dimmer_exit_called, 1);
        assert_eq!(data.borrow().on_exit_called, 1);
        assert_eq!(data.borrow().off_enter_called, 1);
        assert_eq!(data.borrow().off_start_called, 1);
        data.borrow_mut().clear_counts();
    }
    // Change our state back to on!
    {
        light_hsm
            .dispatch_into_hsm(LightEvents::TurnOn)
            .expect("Error dispatching TurnOn event into hsm");

        let state_id = light_hsm.get_current_state();
        let expected_state_id = LightStates::ON;

        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id.as_ref(),
            state_id.as_ref()
        );

        let data = light_hsm.get_light_data();
        assert_eq!(data.borrow().top_enter_called, 0);
        assert_eq!(data.borrow().top_start_called, 0);
        assert_eq!(data.borrow().top_exit_called, 0);
        assert_eq!(data.borrow().on_enter_called, 1);
        assert_eq!(data.borrow().on_start_called, 1);
        assert_eq!(data.borrow().on_exit_called, 0);
        assert_eq!(data.borrow().dimmer_enter_called, 0);
        assert_eq!(data.borrow().dimmer_start_called, 0);
        assert_eq!(data.borrow().dimmer_exit_called, 0);
        assert_eq!(data.borrow().off_enter_called, 0);
        assert_eq!(data.borrow().off_start_called, 0);
        assert_eq!(data.borrow().off_exit_called, 1);
        data.borrow_mut().clear_counts();
    }
    // Cause a no-op, we are already on! - Test an un-handled event
    {
        light_hsm
            .dispatch_into_hsm(LightEvents::TurnOn)
            .expect("Error dispatching TurnOn event into hsm");

        assert!(light_hsm.get_light_data().borrow().none_called());
    }
    // Cause a state change via turn off event (by levering parent behavior!)
    {
        light_hsm
            .dispatch_into_hsm(LightEvents::TurnOff)
            .expect("Error dispatching TurnOff event into HS<");

        let state_id = light_hsm.get_current_state();
        let expected_state_id = LightStates::OFF;

        let data = light_hsm.get_light_data();
        assert_eq!(data.borrow().on_exit_called, 1);
        assert_eq!(data.borrow().off_enter_called, 1);
        assert_eq!(data.borrow().off_start_called, 1);

        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id.as_ref(),
            state_id.as_ref()
        );
        data.borrow_mut().clear_counts();
    }
}
