//! Save and load an audio file.

use fundsp::hacker32::*;

fn main() {
    let wave1 = Wave::render(44100.0, 10.0, &mut (pink()));

    let path = std::path::Path::new("test.wav");

    wave1.save_wav32(path).expect("Could not save test.wav");

    let wave2 = Wave::load(path).expect("Could not load test.wav");

    assert_eq!(wave1.sample_rate(), wave2.sample_rate());
    assert_eq!(wave1.channels(), wave2.channels());
    assert_eq!(wave1.len(), wave2.len());
    assert_eq!(wave1.at(0, 0), wave2.at(0, 0));

    println!("OK.");
}
