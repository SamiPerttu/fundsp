/// This example utility "unscrambles" FunDSP type names reported from the compiler.
///
/// For example,
/// cargo run --example type -- "An<Bus<f64, Noise<f64>, Pipe<f64, fundsp::audionode::Constant<typenum::uint::UInt<typenum::uint::UTerm, typenum::bit::B1>, f64>, Sine<f64>>>>"
/// prints
/// An<Bus<f64, Noise<f64>, Pipe<f64, Constant<U1, f64>, Sine<f64>>>>

fn remove_string(text: &mut String, what: &str) {
    while let Some(position) = text.find(what) {
        for _ in 0..what.len() {
            text.remove(position);
        }
    }
}

fn parse_uint(text: &mut String) {
    while let Some(position) = text.find("UInt") {
        let mut brackets = 1;
        let mut i = position + 5;
        let mut number = 0;
        let bytes = text.as_bytes();
        while brackets > 0 {
            if bytes[i] == b'<' {
                brackets += 1;
            }
            if bytes[i] == b'>' {
                brackets -= 1;
            }
            if bytes[i] == b'1' {
                number = (number << 1) + 1;
            }
            if bytes[i] == b'0' {
                number <<= 1;
            }
            i += 1;
        }
        for _ in position + 1..i {
            text.remove(position + 1);
        }
        text.insert_str(position + 1, format!("{}", number).as_str());
    }
}

fn main() {
    let mut arg: String = std::env::args().nth(1).unwrap_or("You must provide a type.".to_string());

    remove_string(&mut arg, "fundsp::");
    remove_string(&mut arg, "audionode::");
    remove_string(&mut arg, "audiounit::");
    remove_string(&mut arg, "buffer::");
    remove_string(&mut arg, "combinator::");
    remove_string(&mut arg, "delay::");
    remove_string(&mut arg, "dynamics::");
    remove_string(&mut arg, "envelope::");
    remove_string(&mut arg, "feedback::");
    remove_string(&mut arg, "filter::");
    remove_string(&mut arg, "fir::");
    remove_string(&mut arg, "follow::");
    remove_string(&mut arg, "math::");
    remove_string(&mut arg, "moog::");
    remove_string(&mut arg, "net::");
    remove_string(&mut arg, "noise::");
    remove_string(&mut arg, "oscillator::");
    remove_string(&mut arg, "oversample::");
    remove_string(&mut arg, "pan::");
    remove_string(&mut arg, "prelude::");
    remove_string(&mut arg, "realnet::");
    remove_string(&mut arg, "realseq::");
    remove_string(&mut arg, "resample::");
    remove_string(&mut arg, "rez::");
    remove_string(&mut arg, "sequencer::");
    remove_string(&mut arg, "setting::");
    remove_string(&mut arg, "shape::");
    remove_string(&mut arg, "signal::");
    remove_string(&mut arg, "snoop::");
    remove_string(&mut arg, "svf::");
    remove_string(&mut arg, "system::");
    remove_string(&mut arg, "wave::");
    remove_string(&mut arg, "wavetable::");
    remove_string(&mut arg, "typenum::uint::");
    parse_uint(&mut arg);

    println!("\n{}", arg);
}
