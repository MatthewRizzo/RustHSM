# TODO for v2

Main Aims:

* improve ease of use
* Reduce complexity of data structures
* Cleanup any bugs

## Action Items (AI)

1. [ ] Apply clippy suggestions - do last!
2. [ ] Add actual tests!
   1. [x] state_engine_channel_delegate.rs
   2. [ ] state_engine.rs
   3. [x] state_mapping.rs
   4. [x] utils.rs
3. [ ] Define macro to do boilerplate code for impl of state enum trait(s)
4. [ ] Devise a method for the states to un-prompted fire events back at the engine.
   1. Right now we drive all events to completion once prompted by an external force
   2. However, if a state async/unprompted sends an event, the hsm will not wake up to it
   3. The quick-and-dirty solution would be:
      1. The states to somehow loop the call-back to an external source
      2. The external source could then fire back into the hsm
