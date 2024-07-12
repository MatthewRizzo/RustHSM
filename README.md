# RustHSM

Rust implementation of a Hierarchical State Machine (HSM).
It is a library that can be applied to any problem set.
It breaks the HSM into these notable components:

[HsmController]: ./rust_hsm/src/state_controller_trait.rs
[StateChainOfResponsibility]: ./rust_hsm/src/state.rs
[StateEventsIF]: ./rust_hsm/src/events.rs

1. State Controller trait: [HsmController]
2. Concrete Controller implementing [HsmController]: [HSMEngine](./rust_hsm/src/state_controller.rs)
   1. Notably there is very little inside the concrete controller, by design!
   2. If you would like to implement a thread safe controller, copy `HSMEngine` and implement a different `dispatch_event`!
3. States which implement the [StateChainOfResponsibility] trait
   1. Handles flow into / out of a given state
   2. Allows implementers to delegate the handling of their custom enum events within the `handle_event` impl.
4. Event traits dispatched to the HSM controller: [StateEventsIF]
   1. Your event just needs to implement the trait to slot into the larger system.

There is some boiler plate to setup for these pieces when implementing a new HSM
(macro's to come!).
But once that is out of the way, you can add / grow the number of states and events trivially!

## Examples and Usage

Please look at the example [LightHsm](./example_hsm/) to see how the library
can be consumed!

## Considerations

When events have data, you (the implementer) must manage the memory layout of
those arguments yourself.
You have to find a way to serialize/represent that data as a buffer of `[u8]`
That can be understood by [StateEventsIF] and `HsmEvent`.

Some convenient methods might be to use [serde](https://serde.rs/) or protobufs
to serialize & deserialize at HSM boundaries.
