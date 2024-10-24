# Peterson's Algorithm
Implementing Peterson's Algorithm for multiple processes in Rust.



## Implementation Details
### Memory Model
Since to properly test Peterson's Algorithm we need actual unprotected memory, we create a function that takes a `T`, leaks it to be a `&'static mut T` and then cast that to `*mut T`. We can then create multiple multiple references as follows:
```rust
let ptr: *mut T = // ...
let instance: &mut T = unsafe { &mut *ptr }
```
This allows the possibility of data races between threads, which is exactly what we want. In `C` we would just pass pointers to the threads, but Rust is very strict about preventing this behaviour hence why we need to do the above to get around it.

### Detecting Data Races
To detect data races in the critical section, we have a `HotPotato` that is held and released. Underneath, this is just an atomic boolean value, so if it is already being held then it will just `panic!` and crash the program.

There is a small delay added between holding and releasing the potato (in other words, we hold the critical section for a certain period of time). To see why, consider if all the threads execute on one core, it may look like this (with Peterson's disabled):
```
Thread 0
Thread 0
Thread 1
Thread 1
...
```
This is because if it is on a single core and there is no sleep, the processor will not yield to other threads. The delay allows us to actually experience a contended critical section:
```rust
// Thread 1             |  // Thread 2
pot.hold();             |
                        |  // Will crash
                        |  pot.hold()
// may deadlock         |
pot.release()           |
```

### Spin locks
To prevent the example from being overcomplicated, we just use spin locks. These sleep the threads for a small amount of time. Without this the program performs really inefficiently as the processor cannot yield properly to other threads.