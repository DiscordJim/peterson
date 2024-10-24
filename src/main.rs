use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread, time::Duration};


pub const ROUNDS: usize = 10;
pub const PROCESSES: usize = 10;


/// How many milliseconds should each process hold
/// the critical section for?
/// 
/// Note, this should be at least one, or else
/// if all the cores operate on the same prcess
/// this will not properly trip petersons.
pub const CRITICAL_SECTION_TIME_MS: u64 = 1;


/// Spin lock waiting time, this can make the process
/// a little bit more efficient as it prevents the costly
/// checking happening which prevents the OS from rescheduling
/// the threads.
/// 
/// This allows for hundreds of processes to compete.
pub const SPIN_LOCK_WAITING_TIME_MS: u64 = 1;

/// To watch the program experience data races
/// turn this off.
pub const ENABLE_PETERSON: bool = true;

/// Allows us to check that there are no races.
/// 
/// This just maintains an atomic boolean that is
/// written with sequential consistency guarantees.
/// 
/// If we ever true to hold without releasing, it will
/// immediately panic the program.
#[derive(Clone, Default)]
pub struct HotPotato {
    counter: Arc<AtomicBool>
}

impl HotPotato {
    /// Entering the critical section.
    pub fn hold(&self) {
        if self.counter.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            panic!("Potato was already held! Race condition!");
        }
    }
    /// Exiting the critical section.
    pub fn release(&self) {
        if self.counter.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            panic!("Potato was released but not already held! Race condition!");
        }
    }
}

/// For some process i, this checks there is no process j such that
/// j is competing at a higher level if it is i's turn to be scheduled.
pub fn exists(i: usize, k: usize, flag: &mut [i32; PROCESSES], turn: &mut [usize; PROCESSES - 1]) -> bool {
    let mut does_exist = false;
    for j in 0..PROCESSES {
        // for j != i
        if j == i {
            continue;
        } else if flag[j] >= k as i32 && turn[k] == i {
            does_exist = true;
            break;
        }
    }
    does_exist
}

pub fn contender(i: usize, flag: &mut [i32; PROCESSES], turn: &mut [usize; PROCESSES - 1], potato: HotPotato) {
   
    for k in 0..PROCESSES - 1 {
        // Process i is competing at level k
        flag[i] = k as i32;
        // I believe it is my turn at level k
        turn[k] = i;
        
        
        // Is there a process competing at higher levels and at the same level?
        while ENABLE_PETERSON && exists(i, k, flag, turn) {
            // Busy wait, this is just a spin lock.
            // Highly inefficient-- although will hand
            // over control really fast. We can really easily improve this
            // by just putting a condvar here but that would 
            // complicate the example.
            thread::sleep(Duration::from_millis(SPIN_LOCK_WAITING_TIME_MS));
        }

        
    }

    println!("Holding ({})", i);
    potato.hold();
    thread::sleep(Duration::from_millis(CRITICAL_SECTION_TIME_MS));
    potato.release();

    // Process i is no longer competing.
    flag[i] = -1;
}


/// This function creates a mutable pointer by leaking
/// the value.
/// 
/// This value will never be dropped and it is very easy to
/// create data races like this.
/// 
/// Since we are testing mutual exclusion, this is desired.
pub fn create_shared_memory<T>(val: T) -> *mut T {
    Box::leak(Box::new(val)) as *mut T
}


fn main() {

    // This allows us to check that we are doing memory sharing correctly.
    let potato = HotPotato::default();
    
    // Create raw unprotected shared memory.
    let flag: *mut [i32; PROCESSES] = create_shared_memory([-1i32; PROCESSES]);
    let turn: *mut [usize; PROCESSES - 1] = create_shared_memory([0usize; PROCESSES - 1]);

    let mut handles = vec![];

    // Spawn all the threads.
    for i in 0..PROCESSES {
        handles.push(thread::spawn({
            // This operation is HIGHLY unsafe. We are
            // creating pointers that can easily race.
            let flag = unsafe { &mut *flag };
            let turn = unsafe { &mut *turn };
            let potato = potato.clone();
            move || {
                for _ in 0..ROUNDS {
                    contender(i, flag, turn, potato.clone());
                }
            }
        }));
    }

    // Wait for all threads to terminate.
    for handle in handles {
        handle.join().unwrap();
    }

    // Free up memory.
    unsafe {
        std::ptr::drop_in_place(flag);
        std::ptr::drop_in_place(turn);
    }
    
    println!("Finished!");
}
