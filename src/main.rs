//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness source
//! - Debiaser (von Neumann whitening?)
//! - Cryptographically secure pseudorandom number generator (CSPRG)

extern crate rand;

use std::thread;
use std::sync::mpsc;

use rand::Rng;

fn main() {
    let (randomness_transmitter, randomness_receiver) = mpsc::channel();

    let randomness_transmitter = randomness_transmitter.clone();

    thread::spawn(move || loop {
        let mut rng = rand::thread_rng();
        let bit: bool = rng.gen() && rng.gen(); // biased generator
        println!("biased: {}", bit);
        match randomness_transmitter.send(bit) {
            Ok(_) => {},
            Err(_) => {break;}
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
                Ok(_) => {},
                Err(_) => {break;}
            }
        }
    });

    for _ in 0..16 {
        let bit = unbiased_receiver.recv().unwrap();
        println!("unbiased: {}", bit);
    }
}
