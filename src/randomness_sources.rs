//! A collection of boolean streams for use as sources of randomness.

use futures::stream::Stream;
use futures::{Async, Poll};
use rand::{OsRng, Rng};

use std::process::Command;

// A stream which uses the mouse cursor location as a source of randomness.
#[allow(dead_code)]
pub struct MouseStream {}

#[allow(dead_code)]
impl MouseStream {
    pub fn new() -> Self {
        MouseStream {}
    }
}

impl Stream for MouseStream {
    type Item = bool;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // get the output of the xdotool, which includes the mouse cursor location
        let output = Command::new("xdotool")
            .arg("getmouselocation")
            .output()
            .expect("failed to find mouse position");
        let mouse_position = output.stdout;

        // XOR-fold the output; easier than parsing and just as good
        let xor_fold = mouse_position.iter().fold(0, |acc, v| acc ^ v);
        Ok(Async::Ready(Some(xor_fold % 2 == 0)))
    }
}

// A stream which cheats by using the operating system's own source of randomness.
pub struct RandStream {
    rng: OsRng,
}

impl RandStream {
    pub fn new() -> Self {
        RandStream { rng: OsRng::new().expect("Could not initialize OS RNG.") }
    }
}

impl Stream for RandStream {
    type Item = bool;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(Some(self.rng.gen())))
    }
}

// A stream which cheats by using the operating system's own source of randomness.
// Is biased toward a distributation that is 75% false and 25% true.
#[allow(dead_code)]
pub struct BiasedRandStream {
    rng: OsRng,
}

#[allow(dead_code)]
impl BiasedRandStream {
    pub fn new() -> Self {
        BiasedRandStream { rng: OsRng::new().expect("Could not initialize OS RNG.") }
    }
}

impl Stream for BiasedRandStream {
    type Item = bool;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // generate two booleans and take the product to bias the stream toward `false`
        let b1 = self.rng.gen();
        let b2 = self.rng.gen();
        let product = b1 && b2;

        Ok(Async::Ready(Some(product)))
    }
}

// A stream which alternates between `true` and `false`.
#[allow(dead_code)]
pub struct SawtoothStream {
    switch: bool,
}

#[allow(dead_code)]
impl SawtoothStream {
    pub fn new() -> Self {
        SawtoothStream {switch: false}
    }

    fn with(switch: bool) -> Self {
        SawtoothStream {switch: switch}
    }
}

impl Stream for SawtoothStream {
    type Item = bool;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.switch = !self.switch;
        Ok(Async::Ready(Some(self.switch)))
    }
}

// A constant boolean stream.
// Note: von Neumann debiasing on this stream will cause random output to block forever.
#[allow(dead_code)]
pub struct ConstantStream {
    constant: bool,
}

#[allow(dead_code)]
impl ConstantStream {
    pub fn new() -> Self {
        ConstantStream {constant: true}
    }

    fn with(constant: bool) -> Self {
        ConstantStream {constant: constant}
    }
}

impl Stream for ConstantStream {
    type Item = bool;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(Some(self.constant)))
    }
}
