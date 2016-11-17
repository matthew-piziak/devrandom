//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness sources
//! - Mixer (omit for now)
//! - Debiaser (von Neumann?)
//! - Cryptographically secure pseudorandom number generator (CSPRG)
//! - Entropy Counter

extern crate rand;

use std::sync::mpsc;
use std::thread;

use rand::Rng;

fn main() {
    let (randomness_transmitter, randomness_receiver) = mpsc::channel();

    let randomness_transmitter = randomness_transmitter.clone();

    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            let bit: bool = rng.gen() && rng.gen(); // biased boolean generator
            match randomness_transmitter.send(bit) {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
    });

    let (unbiased_transmitter, unbiased_receiver) = mpsc::channel();

    thread::spawn(move || loop {
        let i = randomness_receiver.recv().unwrap();
        let j = randomness_receiver.recv().unwrap();
        if i == j {
            continue;
        } else {
            match unbiased_transmitter.send(i) {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
    });

    loop {
        let mut byte: u8 = 0;
        for _ in 0..8 {
            byte <<= 1;
            if unbiased_receiver.recv().unwrap() {
                byte += 1;
            };
        }
        use std::io::{self, Write};
        io::stdout().write(&[byte]);
        io::stdout().flush();
    }
}
