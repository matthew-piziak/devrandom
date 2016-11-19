//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness sources
//! - Mixer (omit for now)
//! - Debiaser (von Neumann?)
//! - Cryptographically secure pseudorandom number generator (CSPRG)
//! - Entropy Counter

// Note: I'm using a guided-tour style of documentation in this project. Here
// are some reasons why:
// - Since the readers (you!) may not be familiar with Rust, it is more
//   important that I explain the features and idiosyncracies of the language.
// - The readers will probably be reading this package from top to bottom,
//   rather than jumping to specific items in their IDE.
// - This package is an ad-hoc demonstration that is not likely to be updated,
//   so the documentation does not need to be easy to maintain. (Naturally,
//   maintainability of the codebase is something to be judged by. This
//   disclaimer only pertains to the documentation, where I believe the
//   tradeoffs are worth it.)

extern crate futures;
extern crate rand;
extern crate tiny_keccak;

use futures::Future;
use futures::stream::{self, Stream, BoxStream};

use rand::Rng;
use tiny_keccak::Keccak;

type BitStream = BoxStream<bool, ()>;
type ByteStream = BoxStream<u8, ()>;
type HashedStream = BoxStream<[u8; 32], ()>;

fn main() {
    let randomness_stream = mock_randomness_source();
    dev_random(randomness_stream);
}

fn emit(bytestream: HashedStream) {
    let result = bytestream.for_each(emit_item);
    let _ = result.wait();
}

fn emit_item(item: [u8; 32]) -> Result<(), ()> {
    use std::io::{self, Write};
    io::stdout().write(&item).unwrap();
    io::stdout().flush().unwrap();
    Ok(())
}


fn mock_randomness_source() -> BitStream {
    let mut rng = rand::thread_rng();
    let mut rng2 = rand::thread_rng();
    let rng_size = 100_000_000;
    let random_source: Vec<Result<bool, ()>> = rng.gen_iter()
        .zip(rng2.gen_iter())
        .map(|(x, y)| x && y)
        .take(rng_size)
        .map(Ok)
        .collect();
    Box::new(stream::iter(random_source))
}

fn dev_random(randomness_source: BitStream) {
    let results: HashedStream = Box::new(randomness_source.chunks(2)
        .map(vec_to_pair)
        .filter_map(von_neumann_debias)
        .chunks(8)
        .filter_map(octet_to_byte)
        .chunks(32)
        .filter_map(sha3));
    emit(results);
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
