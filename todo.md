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
4. [ ] Devise a method for the hsm to be woken up if anyone puts a request on the queue
   1. [ ] Probably requires moving ALL requests to the channel
   2. [ ] Maybe a notification mechanism waking up the HSMEngine to start processing the events
   3. [ ] For example tokio [Notify](https://docs.rs/tokio/latest/tokio/sync/struct.Notify.html)
   4. Delegates / public API's of the HSMEngine will notify
   5. The HSMEngine will await the notification when idle
   6. Why? Right now, states cannot asynchronously fire events back at the hsm.
      1. They can do so while processing, but not after.
      2. In otherwords, they would need to orchestrate a re-direction of async events externally
      3. If they do not do this, it will not be handled until the NEXT time an external consumer fires
      4. Moreover, it will be handled out of order


* dispatch_event
* 