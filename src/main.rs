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
use futures::stream::{self, Stream, IterStream};
use rand::Rng;
use tiny_keccak::Keccak;

fn main() {
    let randomness_source = mock_randomness_source();
    dev_random(randomness_source);
}

fn mock_randomness_source() -> IterStream<std::vec::IntoIter<Result<bool, ()>>> {
    let mut rng = rand::thread_rng();
    let mut rng2 = rand::thread_rng();
    let rng_size = 100_000_000;
    let random_source: Vec<Result<bool, ()>> = rng.gen_iter()
                                                  .zip(rng2.gen_iter())
                                                  .map(|(x, y)| x && y)
                                                  .take(rng_size)
                                                  .map(Ok)
                                                  .collect();
    stream::iter(random_source)
}

fn dev_random<RandomnessSource>(randomness_source: RandomnessSource)
    where RandomnessSource: futures::Stream<Item = bool>
{
    let results = randomness_source.chunks(2)
                                   .map(vec_to_pair)
                                   .filter_map(von_neumann_debias)
                                   .chunks(8)
                                   .filter_map(octet_to_byte)
                                   .chunks(32)
                                   .filter_map(sha3)
                                   .collect();
    if let Ok(result) = results.wait() {
        for output in result {
            use std::io::{self, Write};
            let _ = io::stdout().write(&output);
            let _ = io::stdout().flush();
        }
    }
}

// Note: the Rust development team has a fairly late stabilization planned for
// the genericization of arrays over length. For now we perform this
// verification at runtime.
fn vec_to_pair<T>(mut vec: Vec<T>) -> (T, T) {
    if vec.len() != 2 {
        panic!("Chunk cannot be converted to a pair");
    } else {
        let x = vec.pop().unwrap();
        let y = vec.pop().unwrap();
        (y, x)
    }
}

fn von_neumann_debias((b1, b2): (bool, bool)) -> Option<bool> {
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
