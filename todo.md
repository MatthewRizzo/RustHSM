# TODO for v2

Main Aims:

* improve ease of use
* Reduce complexity of data structures
* Cleanup any bugs

## Action Items (AI)

1. [x] Use swap to replace change state request in `get_and_reset_requested_state_change`
2. [x] Remove controller from states
   1. Kind of gross that it is there right now. Not sure if used.
3. [x] Try to remove the `Rc`'s / `RefCell's` if possible.
   1. Now that we know the impl works, try to minimize as much of the "cludge" as possible
4. [ ] Add actual tests!
   1. [ ] state.rs
   2. [ ] state_engine_channel_delegate.rs
   3. [ ] events.rs
   4. [ ] state_engine.rs
   5. [x] state_mapping.rs
   6. [ ] utils.rs
5. [x] De-couple `StateChainOfResponsibility` from `StateEngineDelegate` (if possible)
6. [x] Optimize number of borrows that occur
7. [x] Convert u8 buffer of `event_args` to a string that is serialized and deserialized
   1. Done by refactoring the entire event system to better employ enums.
8. [x] Remove `get_state_data` from `StateIF` - Not sure possible!
9. [ ] Attempt to remove dynamic dispatch of StateIF via StateBox.
    1. Now that StateIF is less cluttered, try to generalize HsmController.
    2. Maybe with Create handler map in HSM controller?
10. [x] Deprecate `StateChainOfResponsibility` if possible.
    1. Play around with giving `HSMController` new members to replace it:
       1. state_map: `map<StateId, StateTrait`>
       2. state_chains: `map<StateId, parentStateId>`
          1. We might be able to remove the entire chain! - **YES this worked**
       3. Chain mappings can be created during `AddState(StateBox, Optional parentStateId)`
11. [x] Reduce the number of `expect`. Use results instead!
12. [ ] Try replacing some `format!` with `write!` if it improves performance
13. [ ] Define macro to do boilerplate code for impl of state enum trait(s)
14. [x] Controller and every state should share the same delegate!..Duh!
    1. Only possible b/c state ref might be possible to remove from delegate now.
15. [x] Refactor delegate to use channels
    1. Engine builder creates (tx, rx). Provides rx to engine which owns it
    2. Engine builder exposes `get_delegate(state_id) -> Delegate`
       1. engine builder will provide a delegate for a given id ONE time
       2. It will also validate that the ID is valid for the state type
    3. Delegate acts as a wrapper around `tx` clone
       1. It's API's are `change_state` and `fire_event`
       2. Internally it knows how to format requests on it's tx for that request
       3. Delegate is NOT clone-able
    4. Engine reads the rx queue when it is done handling an event and handles until done
16. Can `StateIF` be embedded within the `LightStates` enum options?
    1. Can we get a state's dyn from the enum variant? Probably not without shared ownership.
