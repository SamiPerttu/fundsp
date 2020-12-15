//! This is a regular crate doc comment, but it also contains a partial
//! Cargo manifest.  Note the use of a *fenced* code block, and the
//! `cargo` "language".
//!
//! ```cargo
//! [dependencies]
//! fundsp = { path = "D:/rust/fundsp" }
//! ```

// Note. To use this as a cargo eval script that accepts filter expressions,
// set the fundsp dependency above to your version or path, then uncomment the #{prelude} line
// and replace "let expression..." and "let node..." lines with the commented out ones.

#![allow(unused_imports)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::needless_range_loop)]
//#{prelude}

extern crate fundsp;

use fundsp::hacker::*;

fn main() {
    let expression = "lowpass_hz(1000.0, 0.5)";
    let node = lowpass_hz(1000.0, 0.5);
    //let expression = "#{script}";
    //let node = #{script};

    let scope = [
        b"------------------------------------------------",
        b"                                                ",
        b"------------------------------------------------",
        b"                                                ",
        b"------------------------------------------------",
        b"                                                ",
        b"------------------------------------------------",
        b"                                                ",
        b"------------------------------------------------",
        b"                                                ",
        b"------------------------------------------------",
        b"                                                ",
        b"------------------------------------------------",
    ];

    let mut scope: Vec<_> = scope.iter().map(|x| x.to_vec()).collect();

    let f: [f64; 48] = [
        10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 120.0, 140.0, 160.0, 180.0,
        200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 600.0, 700.0, 800.0, 900.0, 1000.0,
        1200.0, 1400.0, 1600.0, 1800.0, 2000.0, 2500.0, 3000.0, 3500.0, 4000.0, 4500.0, 5000.0,
        6000.0, 7000.0, 8000.0, 9000.0, 10000.0, 12000.0, 14000.0, 16000.0, 18000.0, 20000.0,
        22000.0,
    ];

    let r: Vec<_> = f
        .iter()
        .map(|&f| (node.response_db(0, f).unwrap(), f))
        .collect();

    let epsilon_db = 1.0e-2;
    let max_r = r.iter().fold((-f64::INFINITY, None), {
        |acc, &x| {
            if abs(acc.0 - x.0) <= epsilon_db {
                (max(acc.0, x.0), None)
            } else if acc.0 > x.0 {
                acc
            } else {
                (x.0, Some(x.1))
            }
        }
    });
    let max_db = ceil(max_r.0 / 10.0) * 10.0;

    for i in 0..f.len() {
        let row = (max_db - r[i].0) / 5.0;
        let mut j = floor(row) as usize;
        let mut c = if row - floor(row) <= 0.5 { b'*' } else { b'.' };
        while j < scope.len() {
            scope[j][i] = c;
            j += 1;
            c = b'*';
        }
    }

    println!();
    let mut row = 0;
    for ascii_line in scope {
        let line = String::from_utf8(ascii_line).unwrap();
        if row & 1 == 0 {
            let db = round(max_db - row as f64 * 5.0) as i64;
            println!("{:3} dB {} {:3} dB", db, line, db);
        } else {
            println!("       {}", line);
        }
        row += 1;
    }

    println!("       |   |    |    |     |    |    |     |    |    |");
    println!("       10  50   100  200   500  1k   2k    5k   10k  20k Hz");

    println!();
    println!("Filter expression:");
    println!("{}", expression);

    println!();
    println!("Peak magnitude:");
    print!("{:.2} dB", max_r.0);
    match max_r.1 {
        Some(f) => {
            println!(" ({:.1} Hz)", f);
        }
        _ => {
            println!();
        }
    }

    println!();
    println!("Footprint:");
    println!("{} bytes", core::mem::size_of_val(&node));

    println!();
    println!("Latency:");
    println!("{:.1} samples", node.latency(0).unwrap());
}
