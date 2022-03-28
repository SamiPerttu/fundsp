extern crate fundsp;

use fundsp::hacker::*;

fn main() {
    let node = lowpass_hz(1000.0, 0.5);

    println!("Filter: lowpass_hz(1000.0, 0.5)\n");
    // The Debug implementation prints an ASCII oscilloscope and other information about the node.
    println!("{:?}", node);
}
