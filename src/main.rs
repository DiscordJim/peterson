use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar}, thread, time::Duration};


pub const ROUNDS: usize = 100;
pub const PROCESSES: usize = 4;

/// Allows us to check that there are no races.
#[derive(Clone, Default)]
pub struct HotPotato {
    counter: Arc<AtomicBool>
}

impl HotPotato {
    pub fn hold(&self) {
        if self.counter.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            panic!("Potato was already held! Race condition!");
        }
    }
    pub fn release(&self) {
        if self.counter.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            panic!("Potato was released but not already held! Race condition!");
        }
    }
}

// pub fn process_one(flag: &mut Box<[bool; 2]>, turn: &mut Box<usize>, potato: HotPotato) {

    
//     for _ in 0..ROUNDS {
//         flag[0] = true;
//         **turn = 1;
//         while flag[1] && **turn == 1 {
//             // busy wait
//         }
//         potato.hold();

//         //critical
//         println!("P1 Critical.");

//         potato.release();

//         flag[0] = false;

//     }
   
// }

// pub fn process_zero(flag: &mut Box<[bool; 2]>, turn: &mut Box<usize>, potato: HotPotato) {
//     for _ in 0..ROUNDS {
//         flag[1] = true;
//         **turn = 0;

//         while flag[0] && **turn == 0 {

//         }

//         // Enter Critical
//         potato.hold();


//         potato.release();
//         // End Critical
//         flag[1] = false;
//     }
// }

pub fn exists(i: usize, k: usize, flag: &mut Box<[i32; PROCESSES]>, turn: &mut Box<[usize; PROCESSES - 1]>) -> bool {
    let mut does_exist = false;
    for j in 0..PROCESSES {
        if j == i {
            // for j != i
            continue;
        }
        if flag[j] >= k as i32 && turn[k] == i {
            does_exist = true;
            break;
        }
    }
    does_exist
}

pub fn contender(i: usize, flag: &mut Box<[i32; PROCESSES]>, turn: &mut Box<[usize; PROCESSES - 1]>, potato: HotPotato) {
   
    for k in 0..PROCESSES - 1 {
        flag[i] = k as i32;
        turn[k] = i;
        
        
        while exists(i, k, flag, turn) {
            // Busy wait
        }

        
    }

    println!("Holding ({})", i);
    potato.hold();
    thread::sleep(Duration::from_millis(10));
    potato.release();
    flag[i] = -1;
}






fn main() {

    let potato = HotPotato::default();

    
    

    let mut flag = Box::new([-1i32; PROCESSES]);
    let mut turn = Box::new([0usize; PROCESSES - 1]);

    let flag_ptr = (&mut flag) as *mut Box<[i32; PROCESSES]>;
    let turn_ptr = (&mut turn) as *mut Box<[usize; PROCESSES - 1]>;

    let mut handles = vec![];

    for i in 0..PROCESSES {
        handles.push(thread::spawn({

            let flag = unsafe { &mut *flag_ptr };
            let turn = unsafe { &mut *turn_ptr };
            let potato = potato.clone();
            move || {
                for _ in 0..ROUNDS {
                    contender(i, flag, turn, potato.clone());
                }
                
                // process_zero(flag, turn, potato);
            }
        }));
    }

    // let handler2 = thread::spawn({
    //     let flag = unsafe { &mut *flag_ptr };
    //     let turn = unsafe { &mut *turn_ptr };
    //     let potato = potato.clone();
    //     || {
    //         process_one(flag, turn, potato);
    //     }
    // });

    for handle in handles {
        handle.join().unwrap();
    }



    // handler.join().unwrap();
    // handler2.join().unwrap();

    
    
    println!("Hello, world!");
}
