//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number generator which gathers
//! randomness from environmental noise.

// Note: I'm using a guided-tour style of documentation in this project. Here are some reasons why:
// - Since the readers (you!) may not necessarily be familiar with Rust, it is more important that
//   I explain the features and idiosyncracies of the language.
// - The readers will probably be reading this package from top to bottom, rather than jumping to
//   specific items in their IDE.
// - This package is an ad-hoc demonstration that is not likely to be updated, so the documentation
//   does not need to be easy to maintain. (Naturally, maintainability of the codebase is something
//   to be judged by. This disclaimer only pertains to the documentation, where I believe the
//   tradeoffs are worth it.)

// allow `impl Trait` syntax in the return position
#![feature(conservative_impl_trait)]

extern crate futures;
extern crate rand;
extern crate tiny_keccak;
extern crate tokio_core;

// In this binary we use the `futures-rs` framework, which provides "zero-cost" futures and
// streams. This allows us to abstractly operate on an asyncronous pipeline that starts with our
// source of randomness and ends with our final output.
use futures::stream::Stream;

// The Tokio "reactor core" is the event loop that polls our streams.
use tokio_core::reactor::Core;

// `kekkak` provides implentations of cryptographic hash functions.
use tiny_keccak::Keccak;

// In `randomness_sources.rs` we provide a handful of boolean streams that can be plugged into our
// entropy generation function.
mod randomness_sources;

fn main() {
    // other sources of randomness can be substituted here
    let randomness_source = randomness_sources::RandStream::new();

    generate_entropy(randomness_source);
}

// Our entropy generation has three components:
// 1. A source of randomness that is (ostensibly) unlikely to be replicable by an adversary.
// 2. A debiaser (a.k.a. whitener, decorrelator)
// 3. A cryptographic hash function to make the internal state of the program difficult to
//    determine from its output.
//
// Note: the implementation of /dev/random & /dev/urandom uses entropy pools, entropy mixing, and
// entropy counting. In this case we are only attempting to clone /dev/random, which blocks on
// insufficient entropy. We simplify the architecture to a one-to-one pipeline which blocks
// implicitly when the source of randomness is blocking.
fn generate_entropy<RandomnessSource>(randomness_source: RandomnessSource)
    where RandomnessSource: Stream<Item = bool, Error = ()>
{
    // initialize the event loop
    let mut core = Core::new().unwrap();

    // debias the source of randomness
    let debiased_randomness_source = debias(randomness_source);

    // pack the debiased stream into chunks of 32 bytes, run those chunks through a cryptographic
    // hash function, and then output them
    core.run(debiased_randomness_source.chunks(8)
            .filter_map(octet_to_byte)
            .chunks(32)
            .filter_map(sha3)
            .for_each(emit_item))
        .expect("Core event loop failed."); // panic at runtime if the event loop fails
}

// We use Von Neumann's whitening algorithm for our debiasing step.
fn debias<RandomnessStream: Stream<Item = bool, Error = ()>>
    (randomness_stream: RandomnessStream)
     -> impl Stream<Item = bool, Error = ()> {
    randomness_stream.chunks(2).map(vec_to_pair).filter_map(von_neumann_debias)
}

// Note: the Rust development team has a fairly late stabilization planned for the genericization
// of arrays over length. For now we perform this verification at runtime.
fn vec_to_pair<T>(mut vec: Vec<T>) -> (T, T) {
    if vec.len() != 2 {
        panic!("Chunk cannot be converted to a pair");
    } else {
        let x = vec.pop().unwrap();
        let y = vec.pop().unwrap();
        (y, x)
    }
}

// Von Neumann debiasing: 01 becomes 0, 10 becomes 1, else skip.
fn von_neumann_debias((b1, b2): (bool, bool)) -> Option<bool> {
    if b1 != b2 { Some(b1) } else { None }
}

// Convert a vector of eight booleans into a `u8` representation.
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

// Hash a vector of 32 bytes with SHA-3.
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

// Print the byte vector to stdout.
fn emit_item(item: Vec<u8>) -> Result<(), ()> {
    use std::io::{self, Write};
    io::stdout().write(&item).and(io::stdout().flush()).map_err(|_| ())
}
