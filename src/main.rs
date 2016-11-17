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

use futures::stream::{self, Stream};
use futures::future::{Future};
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let mut rng2 = rand::thread_rng();
    let random_source = rng.gen_iter().zip(rng2.gen_iter()).map(|(x, y)| x && y).take(100).map(Ok);
    let bool_stream = stream::iter::<_, bool, ()>(random_source);
    let results = bool_stream.chunks(2).filter_map(von_neumann_debiasing).collect();
    for result in results.wait() {
        for bit in result {
            println!("{:?}", bit);
        }
    }
}

fn von_neumann_debiasing(mut bool_pair: Vec<bool>) -> Option<bool> {
    if let Some(b1) = bool_pair.pop() {
        if let Some(b2) = bool_pair.pop() {
            if b1 != b2 {
                Some(b1)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
