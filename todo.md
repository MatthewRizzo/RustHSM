# TODO for v2

Main Aims:

* improve ease of use
* Reduce complexity of data structures
* Cleanup any bugs

## Action Items (AI)

1. [x] Use swap to replace change state request in `get_and_reset_requested_state_change`
2. [x] Remove controller from states
   1. Kind of gross that it is there right now. Not sure if used.
3. [ ] Move `requested_state_change` API to `StateChainOfResponsibility` instead of `StateDataDelegate`
   1. Tie it to API, not a specific data structure
4. [ ] Create a macro called `change_state!(state: &dyn StateChainOfResponsibility)`
   1. so that user's dont have to call it themselves / know how it works.
5. [ ] Try to remove the `Rc`'s / `RefCell's` if possible.
   1. Now that we know the impl works, try to minimize as much of the "cludge" as possible
6. [ ] Add actual tests!
7. [x] De-couple `StateChainOfResponsibility` from `StateDataDelegate` (if possible)
8. [ ] Optimize number of borrows that occur
9. [ ] Convert u8 buffer of `event_args` to a string that is serialized and deserialized
   1. [ ] Can be integrated with serde
10. [ ] Remove `get_state_data` from `StateIF` - Not sure possible!
