//! Save and load an audio file.

use fundsp::hacker32::*;

fn main() {
    let wave = Wave32::render(44100.0, 10.0, &mut (pink()));

    let path = std::path::Path::new("test.wav");

    wave.save_wav32(path).expect("Could not save test.wav");

    let wave2 = Wave32::load(path).expect("Could not load test.wav");

    assert_eq!(wave.channels(), wave2.channels());
    assert_eq!(wave.len(), wave2.len());
    assert_eq!(wave.at(0, 0), wave2.at(0, 0));

    println!("OK.");
}
