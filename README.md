# Peterson's Algorithm
Implementing Peterson's Algorithm for multiple processes in Rust.

## Algorithm Details
The shared memory is as follows:
- There is a `flag` array which contains `[0, ... n - 1]` values. These are initially all `-1`. This indicates what level each process is competing at. This value is set to `-1` to indicate that a process is *no longer competing* and is usually set at the end of the critical section.
- There is a `turn` array which contains `[0, ... n - 2]` values. This indicates what process believes it is their turn at a certain level.

Each process runs a subroutine:
```rust
// (1)
for k in 0..PROCESSES - 1 {
    flag[i] = k as i32; // (1)
    turn[k] = i; // (2)
    
    while exists(i, k, flag, turn) { // (3)
        // Spin (4)
    }

    // Advancing to next level (5)
}

// Enter critical section


// Exit critical section
flag[i] = -1; // (6)

```
The subroutine is described as follows for some process `i`:
1. For `n` processes there are `n-1` levels. For a process to gain access to the critical section it must compete at all these levels.
2. Upon beginning the competition at some level `k`, `i` sets it's flag to `k` to indicate that `i` is competing at level `k`.
3. While there exists a process that is competing at the same level or higher, we wait. We can only advance once there are no processes competing at the same level or higher. Additionally, we check `turn[k] == i` as we want to see if it is our turn at the level. Consider the case where this is false, that means *another process has arrived at our level* and thus we may proceed upwards.
4. This is just a simple spin lock, or it could involve something more complex for a waiting mechanism.
5. We have "won" the competition at this level and can now proceed upwards.
6. After exiting the critical section, we set our flag to `-1` to indicate we are no longer competiting.

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