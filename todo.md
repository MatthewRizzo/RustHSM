# TODO for v2

Main Aims:

* improve ease of use
* Reduce complexity of data structures
* Cleanup any bugs

## Action Items (AI)

1. [ ] Apply clippy suggestions - do last!
2. [ ] Add actual tests!
   1. [ ] state.rs
   2. [ ] state_engine_channel_delegate.rs
   3. [ ] events.rs
   4. [ ] state_engine.rs
   5. [x] state_mapping.rs
   6. [ ] utils.rs
3. [ ] Attempt to remove dynamic dispatch of StateIF via StateBox.
    1. Now that StateIF is less cluttered, try to generalize HsmController.
    2. Maybe with Create handler map in HSM controller?
4. [ ] Try replacing some `format!` with `write!` if it improves performance
5. [ ] Define macro to do boilerplate code for impl of state enum trait(s)
