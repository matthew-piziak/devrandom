//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness sources
//! - Mixer (omit for now)
//! - Debiaser (von Neumann?)
//! - Cryptographically secure pseudorandom number generator (CSPRG)
//! - Entropy Counter

extern crate futures;
extern crate rand;

use std::thread;

use futures::stream::Stream;
use futures::stream;
use futures::task;
use rand::Rng;

fn main() {
    let (randomness_transmitter, randomness_receiver) = stream::channel<bool, bool>();

    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            let bit: bool = rng.gen() && rng.gen(); // biased boolean generator
            randomness_transmitter.send(Ok(bit));
        }
    });

    // let (unbiased_transmitter, unbiased_receiver) = stream::channel();

    // thread::spawn(move || loop {
    //     let i = randomness_receiver.recv().unwrap();
    //     let j = randomness_receiver.recv().unwrap();
    //     if i == j {
    //         continue;
    //     } else {
    //         match unbiased_transmitter.send(i) {
    //             Ok(_) => {}
    //             Err(_) => {
    //                 break;
    //             }
    //         }
    //     }
    // });

    loop {
        let mut byte: u8 = 0;
        for _ in 0..8 {
            byte <<= 1;
            if let Ok(r) = randomness_receiver.poll() {
                byte += 1;
            }
        }
        use std::io::{self, Write};
        io::stdout().write(&[byte]);
        io::stdout().flush();
    }
}
