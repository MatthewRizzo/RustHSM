# RustHSM

Rust implementation of a Hierarchical State Machine library.
It breaks the HSM into these notable components:

[HsmController]: ./rust_hsm/src/state_controller_trait.rs
[StateChainOfResponsibility]: ./rust_hsm/src/state.rs
[StateEventsIF]: ./rust_hsm/src/events.rs

1. State Controller trait: [HsmController]
2. Concrete Controller implementing [HsmController]: [HSMControllerBase](./rust_hsm/src/state_controller.rs)
   1. Notably there is very little inside the concrete controller, by design!
   2. If you would like to implement a thread safe controller, copy `HSMControllerBase` and implement a different `external_dispatch_into_hsm`!
3. States which implement the [StateChainOfResponsibility] trait
   1. Handles flow into/ out of a given state
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
You have to find a way top serialize/represent that data as a buffer of `[u8]`
That can be understood by [StateEventsIF] and `HsmEvent`.

## StateChainOfResponsibility Details

`StateChainOfResponsibility` encapsulates all the controller needs to know
about a state and how they relate to one another.
The controller uses the trait to attempt to handle an event at the current level.
If the event is unhandled for that state (false), it goes up the "chain".
This "chain" is the hierarchy of states.

## Understanding Change State

The controller always has a "current" state, but in the process of handling
events, you can inform the controller of a state change request.

This is done indirectly, by requesting a state change with the `ComposableStateData` held by all states.

Here is an example:

```Rust
pub struct FakeState {
    /// All state's that work with the system will need to have a member of type ComposableStateData
    state_data: ComposableStateData,
    shared_data: MyCustomDataStructureSharedAcrossAllStates,
}

impl StateChainOfResponsibility for FakeState
{
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        let events: FakeEvent = FakeEvent::from(event);
        // top returns true for all events
        match events {
            FakeEvent::Event1 => self.handle_event1(),
            _ => false,
        }
    }
}

impl FakeState {
    fn handle_event1() {
        // do some stuff!
        self.state_data.submit_state_change_request(FakeStateEnum::FakeState2 as u16);
        return true
    }
}
```

## Understanding StateEventsIF and HsmEvent's Memory Layout

The library will not know every set of events & arguments at compile-time.
This is a problem when dealing with Rust's memory layout which NEEDS to know the
EXACT memory layout at compile time.

One would think Generics could work here, and is what I instinctively reached for.
Unfortunately, this does not work because (unlike C++'s interfaces), Rust does
NOT allow containers object sliced traits/interfaces when that trait accepts generics.

In the case of C++:

```C++
struct<T> Foo : public FooIF<T> {
    Foo() :
        FooIF<T>()
    {
        // implementation details
    }
};

void main()
{
    std::vector<FooIF> container;
    container.push_back(Foo<int>());
    container.push_back(Foo<bool>());

    // do stuff
}
```

Rust does not allow that! How can it know the size of `container`'s nodes at
compile time when there are both traits and generics involved? It doesn't.
How does the Rust compiler resolve the fact we have both `FooIF<int>` and `FooIF<bool>` in container?
It doesn't allow it!

(Trust me I banged my head against a wall here for months, but at least I learned something!)

Taking it a step further, assume that we found a way around Rust's intrinsic behavior.

* How would could the HSM controller - defined in a library -
understand the memory layout of ALL possible event's and their arguments?
* How would the controller call functions with signatures it does not know?
  * In the case of function pointers/callbacks to handlers.

It can't and wouldn't! We run into the same fundamental problem of C++ to Rust incompatibility!
You might be able to get away with behavior like this in C++ if done properly,
but you might also have a bug-ridden rats nest of undefined behavior that Rust
will NEVER let you compile with.

So clearly the C++ approach is off the table.

As a result, instead of fighting Rust, this library leans into it.

* Rather than accept event arguments with generics, take event arguments as a
universal data structure: a buffer of `u8`'s!
  * The data of events should be opaque to the controller as it passes through it.
  * It is the responsibility of the HSM event caller to serialize the arguments
  * It is the responsibility of the handler of the HSM event to know the data format of the arguments and de-serialize them.
  * (No this is not super complicated, `serde` could do the job.)
    * The business logic could even be hidden within an impl of your enum!
* The library is NOT responsible for calling your handlers of each specific event.
  * Instead the [HsmController] delegates the handling of the event to `StateChainOfResponsibility::handle_event`
    * Which in-turn, is controlled by the developer!
    * When implementing a concrete state and adding it to the HSM,
  * The library provides an extendable API for state's register handlers at the higher level with idiomatic ENUM match!

```Rust

/// Really recommended to obfuscate the serialization logic of event args
impl From<&dyn StateEventsIF> for FakeEventEnum
{
    fn from(event: &dyn StateEventsIF) -> Self {
        /// Trait impl that returns u16's
        match event.get_event_id() {
            1 => FakeEventEnum::Event1,
            2 => FakeEventEnum::Event2,
            3 => {
                let serialized_args = event.get_args();
                FakeEventEnum::Event3WithArgs(FakeArgData::from_u8_buffer(serialized_args));
            }
        }
    }
}


impl StateChainOfResponsibility for FakeStateFoo {
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        // (it is suggested to implement this from as well)
        let events: FakeEventEnum = FakeEventEnum::from(event);
        match events {
            FakeEventEnum::Event1 => {
                // Imagine it is implemented and returns a bool
                self.handle_event1()
            },
            FakeEventEnum::Event3WithArgs(args: FakeArgData) => {
                // Imagine it is implemented and returns a bool
                self.handle_event3(args)
            },
            /// All other events (Event2) are unhandled, try the parent state!
            _ => false
        }
    }
}
```

* Notice that in `Event3WithArgs` we use a `FakeArgData` rather than a `vec<u8>`?
  * This is possible because of the `From<&dyn StateEventsIF>` we implemented!
* So now whenever `FakeStateFoo` is the current state, the controller will call its `handle_event`
  * Essentially delegating the handling back to the actual state, but acting as a passthrough for the data it needs!
