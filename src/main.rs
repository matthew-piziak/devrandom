//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness source
//! - Debiaser
//! - Hasher

// Note: I'm using a guided-tour style of documentation in this project. Here
// are some reasons why:
// - Since the readers (you!) may not necessarily be familiar with Rust, it is
//   more important that I explain the features and idiosyncracies of the
//   language.
// - The readers will probably be reading this package from top to bottom,
//   rather than jumping to specific items in their IDE.
// - This package is an ad-hoc demonstration that is not likely to be updated,
//   so the documentation does not need to be easy to maintain. (Naturally,
//   maintainability of the codebase is something to be judged by. This
//   disclaimer only pertains to the documentation, where I believe the
//   tradeoffs are worth it.)

#![feature(conservative_impl_trait)]

extern crate futures;
extern crate rand;
extern crate tiny_keccak;
extern crate tokio_core;

use futures::stream::Stream;
use tiny_keccak::Keccak;
use tokio_core::reactor::Core;

mod randomness_sources;

fn main() {
    let randomness_source = randomness_sources::MouseStream::new();
    start(randomness_source);
}

fn start<RandomnessSource>(randomness_source: RandomnessSource)
    where RandomnessSource: Stream<Item = bool, Error = ()>
{
    let mut core = Core::new().unwrap();
    let debiased_randomness_source = debias(randomness_source);
    core.run(debiased_randomness_source.chunks(8)
            .filter_map(octet_to_byte)
            .chunks(32)
            .filter_map(sha3)
            .for_each(emit_item))
        .unwrap();
}

fn debias<RandomnessStream: Stream<Item = bool, Error = ()>>
    (randomness_stream: RandomnessStream)
     -> impl Stream<Item = bool, Error = ()> {
    randomness_stream.chunks(2).map(vec_to_pair).filter_map(von_neumann_debias)
}

fn emit_item(item: Vec<u8>) -> Result<(), ()> {
    use std::io::{self, Write};
    io::stdout().write(&item).unwrap();
    io::stdout().flush().unwrap();
    Ok(())
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
    if b1 != b2 { Some(b1) } else { None }
}

fn octet_to_byte(bool_octet: Vec<bool>) -> Option<u8> {
    if bool_octet.len() != 8 {
        panic!("Not enough bits for a byte");
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

fn sha3(input: Vec<u8>) -> Option<Vec<u8>> {
    if input.len() != 32 {
        panic!("Not enough bytes to hash");
    }
    let mut sha3 = Keccak::new_sha3_256();
    let data: Vec<u8> = From::from(input);
    sha3.update(&data);
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    Some(res.to_vec())
}
