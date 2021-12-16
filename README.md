This is a clone of `$ cat /dev/random`, i.e. a blocking pseudorandom number generator which gathers randomness from
environmental noise. It was written in three hours as an exercise, and should not be used.

# Compilation

This script is written in **Rust**, and can be built with `cargo`. On a
computer with an internet connection, it should be sufficient to invoke `cargo
run` in the project directory.

# Design

In lieu of a top-level design document, I've decided to use a guided-tour style
of documentation in the code itself. `main.rs` is the right place to start
reading.

# Next Steps

With more time I would have:
- added test suites
- added interfaces for swappable debiasers and cryptographic hash functions
- given the mouse-pointer randomness source better cross-platform support

# Verification

I manually verified output with the `ent` utility.

Here is a sample of what I considered satisfactory `ent` output:

    Entropy = 7.999728 bits per byte.

    Optimum compression would reduce the size
    of this 611744 byte file by 0 percent.

    Chi square distribution for 611744 samples is 231.12, and randomly
    would exceed this value 85.60 percent of the times.

    Arithmetic mean value of data bytes is 127.4323 (127.5 = random).
    Monte Carlo value for Pi is 3.136969507 (error 0.15 percent).
    Serial correlation coefficient is -0.000917 (totally uncorrelated = 0.0).

# A Note on Mouse-cursor-generated Randomness

If you replace `RandStream` with `MouseStream` in `main.rs`, you'll have to move
the mouse around for at least a few seconds before seeing output.

This is because:
- the generator is blocking
- the cursor position does not provide a large number of bits
- output from a nonmoving mouse will be thrown away by the debiaser
