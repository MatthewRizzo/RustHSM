# RustHSM
Rust implementation of a Hierarchical State Machine library that supports generic state's and contexts

## TODO

* Macro that takes ComposableStateData and goes through the path to change state

```rust
let state_data: ComposableStateData = // blah blah blah

// now
state_data.get_hsm().borrow_mut().change_state(LightStates::OFF as u16)

// desired
change_state!(state_data, LightStates::OFF as u16)
```
