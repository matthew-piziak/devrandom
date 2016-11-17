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

use futures::stream::{self, Stream};
use futures::future::{finished, Future};
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let random_source = rng.gen_iter().take(100).map(Ok);
    let number_stream = stream::iter::<_, bool, ()>(random_source);
    let sum = number_stream.fold(true, |a, b| finished(a && b));
    println!("sum: {:?}", sum.wait());
}
