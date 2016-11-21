use futures::stream::Stream;
use futures::{Async, Poll};
use rand::{OsRng, Rng};

use std::process::Command;

pub struct MouseStream {}

impl MouseStream {
    pub fn new() -> Self {
        MouseStream {}
    }
}

impl Stream for MouseStream {
    type Item = bool;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let output = Command::new("xdotool")
            .arg("getmouselocation")
            .output()
            .expect("failed to find mouse position");
        let mouse_position = output.stdout;
        let val = mouse_position.iter().fold(0, |acc, v| acc ^ v);
        let b = val % 2 == 0;
        Ok(Async::Ready(Some(b)))
    }
}

#[allow(dead_code)]
pub struct RandStream {
    rng: OsRng,
}

#[allow(dead_code)]
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
        let b1 = self.rng.gen();
        let b2 = self.rng.gen();
        let product = b1 && b2;
        Ok(Async::Ready(Some(product)))
    }
}

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
