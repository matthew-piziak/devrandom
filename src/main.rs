//! A clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number
//! generator which gathers randomness from environmental noise.
//!
//! Architectural components:
//! - Randomness source
//! - Debiaser (von Neumann whitening?)
//! - Cryptographically secure pseudorandom number generator (CSPRG)

extern crate futures;
extern crate tokio_core;

use futures::stream;
use tokio_core::reactor;

fn main() {
    let randomness_source: RandomnessStream = MockRandomnessSource::new(vec![true, false, true])
        .stream();

    randomness_source.for_each(|r| println!("{}", r))
}

type RandomnessStream = stream::BoxStream<bool, RandomnessSourceError>;

enum RandomnessSourceError {}

trait RandomnessSource {
    fn stream(&self) -> RandomnessStream;
}

struct MockRandomnessSource {
    random_data: Vec<bool>,
}

impl MockRandomnessSource {
    fn new(random_data: Vec<bool>) -> Self {
        MockRandomnessSource { random_data: random_data }
    }
}

impl RandomnessSource for MockRandomnessSource {
    fn stream(&self) -> RandomnessStream {
        let results: Vec<Result<bool, RandomnessSourceError>> =
            self.random_data.clone().into_iter().map(Ok).collect();
        Box::new(stream::iter(results))
    }
}
