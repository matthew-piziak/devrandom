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
extern crate tiny_keccak;

use futures::future::Future;
use futures::stream::{self, Stream};
use rand::Rng;
use tiny_keccak::Keccak;

fn main() {
    let mut rng = rand::thread_rng();
    let mut rng2 = rand::thread_rng();
    let rng_size = 100_000_000;
    let random_source = rng.gen_iter()
                           .zip(rng2.gen_iter())
                           .map(|(x, y)| x && y)
                           .take(rng_size)
                           .map(Ok);
    let bool_stream = stream::iter::<_, bool, ()>(random_source);
    let results = bool_stream.chunks(2)
                             .filter_map(von_neumann_debiasing)
                             .chunks(8)
                             .filter_map(octet_to_byte)
                             .chunks(32)
                             .filter_map(sha3)
                             .collect();
    for result in results.wait() {
        for output in result {
            use std::io::{self, Write};
            let _ = io::stdout().write(&output);
            let _ = io::stdout().flush();
        }
    }
}

fn von_neumann_debiasing(mut bool_pair: Vec<bool>) -> Option<bool> {
    if bool_pair.len() != 2 {
        panic!("Von Neumann debiasing requires pairs of size 2");
    }
    let b1 = bool_pair.pop().unwrap();
    let b2 = bool_pair.pop().unwrap();
    if b1 != b2 {
        Some(b1)
    } else {
        None
    }
}

fn octet_to_byte(bool_octet: Vec<bool>) -> Option<u8> {
    if bool_octet.len() != 8 {
        return None;
    }
    let mut byte = 0;
    for b in bool_octet {
        byte <<= 1;
        if b {
            byte += 1;
        }
    }
    Some(byte)
}

fn sha3(input: Vec<u8>) -> Option<[u8; 32]> {
    if input.len() != 32 {
        return None;
    }
    let mut sha3 = Keccak::new_sha3_256();
    let data: Vec<u8> = From::from(input);
    sha3.update(&data);
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    Some(res)
}
