//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness source
//! - Debiaser (von Neumann whitening?)
//! - Cryptographically secure pseudorandom number generator (CSPRG)
//!
//! TODO: How to represent a bitstream in Rust?

extern crate rand;
use rand::Rng;

fn main() {
    let mut entropy_count = 0;
    loop {
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen();
        if rng.gen() {
            entropy_count += 1;
        }
        if entropy_count > 0 {
            entropy_count -= 1;
            println!("{}", random_number);
        } else {
            println!("Blocking");
        }
    }
}
