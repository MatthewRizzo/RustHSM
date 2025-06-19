mod light_hsm;
use futures::executor::block_on;
use light_hsm::{
    light_events, light_events::LightEvents, light_hsm_controller::LightControllerHsm,
    light_hsm_data, light_states, light_states::LightStates,
};

#[tokio::main]
async fn main() {
    let mut light_hsm = LightControllerHsm::new(log::LevelFilter::Info);

    // I am blocking here for simplicity / transparency of the code. but in reality, you should/could use tokio runtimes to await.
    println!("Blocking for get_current_state");
    let starting_state = block_on(light_hsm.get_current_state());
    assert!(
        starting_state == LightStates::DIMMER,
        "starting_state = {}. Expected: {}",
        starting_state,
        LightStates::DIMMER
    );
    println!("Done blocking for get_current_state");

    // Check setup / init. Top -> On -> DIMMER
    {
        let data = light_hsm.get_light_data();
        assert_eq!(data.read().unwrap().top_enter_called, 1);
        assert_eq!(data.read().unwrap().top_start_called, 0);
        assert_eq!(data.read().unwrap().top_exit_called, 0);
        assert_eq!(data.read().unwrap().on_enter_called, 1);
        assert_eq!(data.read().unwrap().on_start_called, 0);
        assert_eq!(data.read().unwrap().on_exit_called, 0);
        assert_eq!(data.read().unwrap().dimmer_enter_called, 1);
        assert_eq!(data.read().unwrap().dimmer_start_called, 1);
        assert_eq!(data.read().unwrap().dimmer_exit_called, 0);
        assert_eq!(data.read().unwrap().off_enter_called, 0);
        assert_eq!(data.read().unwrap().off_start_called, 0);
        assert_eq!(data.read().unwrap().off_exit_called, 0);
        data.write().unwrap().clear_counts();
    }

    println!("Sending Set() into HSM");

    // Set the dimmer value that triggers another internal event being fired for OFF
    light_hsm
        .dispatch_into_hsm(LightEvents::Set(0))
        .await
        .expect("Error dispatching Set event into HSM");

    println!("Waiting to receive current state from hsm");
    let state_id = light_hsm.get_current_state().await;
    println!("Done waiting to receive current state from hsm");
    let expected_state_id = LightStates::OFF;
    assert!(
        state_id == expected_state_id,
        "Expected state id = {}. Found {}",
        expected_state_id.as_ref(),
        state_id.as_ref()
    );

    let data = light_hsm.get_light_data();
    assert_eq!(data.read().unwrap().light_percentage, 0);
    assert_eq!(data.read().unwrap().dimmer_exit_called, 1);
    assert_eq!(data.read().unwrap().on_exit_called, 1);
    assert_eq!(data.read().unwrap().off_enter_called, 1);
    assert_eq!(data.read().unwrap().off_start_called, 1);
    data.write().unwrap().clear_counts();

    // Change our state back to on!
    async {
        light_hsm
            .dispatch_into_hsm(LightEvents::TurnOn)
            .await
            .expect("Error dispatching TurnOn event into hsm");

        let state_id = light_hsm.get_current_state().await;
        let expected_state_id = LightStates::ON;

        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id.as_ref(),
            state_id.as_ref()
        );

        let data = light_hsm.get_light_data();
        assert_eq!(data.read().unwrap().top_enter_called, 0);
        assert_eq!(data.read().unwrap().top_start_called, 0);
        assert_eq!(data.read().unwrap().top_exit_called, 0);
        assert_eq!(data.read().unwrap().on_enter_called, 1);
        assert_eq!(data.read().unwrap().on_start_called, 1);
        assert_eq!(data.read().unwrap().on_exit_called, 0);
        assert_eq!(data.read().unwrap().dimmer_enter_called, 0);
        assert_eq!(data.read().unwrap().dimmer_start_called, 0);
        assert_eq!(data.read().unwrap().dimmer_exit_called, 0);
        assert_eq!(data.read().unwrap().off_enter_called, 0);
        assert_eq!(data.read().unwrap().off_start_called, 0);
        assert_eq!(data.read().unwrap().off_exit_called, 1);
        data.write().unwrap().clear_counts();
    }
    .await;
    // Cause a no-op, we are already on! - Test an un-handled event
    async {
        light_hsm
            .dispatch_into_hsm(LightEvents::TurnOn)
            .await
            .expect("Error dispatching TurnOn event into hsm");

        assert!(light_hsm.get_light_data().read().unwrap().none_called());
    }
    .await;
    // Cause a state change via turn off event (by levering parent behavior!)
    async {
        light_hsm
            .dispatch_into_hsm(LightEvents::TurnOff)
            .await
            .expect("Error dispatching TurnOff event into HS<");

        let state_id = light_hsm.get_current_state().await;
        let expected_state_id = LightStates::OFF;

        let data = light_hsm.get_light_data();
        assert_eq!(data.read().unwrap().on_exit_called, 1);
        assert_eq!(data.read().unwrap().off_enter_called, 1);
        assert_eq!(data.read().unwrap().off_start_called, 1);

        assert!(
            state_id == expected_state_id,
            "Expected state id = {}. Found {}",
            expected_state_id.as_ref(),
            state_id.as_ref()
        );
        data.write().unwrap().clear_counts();
    }
    .await;
}
