//! Interface to microfft.
use microfft::inverse::*;
use microfft::real::*;
use num_complex::Complex32;

/// Perform real-valued FFT in-place.
/// The length of `data` must be a power of two between 2 and 32768.
/// The Nyquist frequency will be packed into the imaginary part of DC (the first element).
/// Returns the slice transmuted into a slice of `Complex32`.
pub fn real_fft(data: &mut [f32]) -> &mut [Complex32] {
    match data.len() {
        2 => rfft_2(data.try_into().unwrap()).as_mut_slice(),
        4 => rfft_4(data.try_into().unwrap()).as_mut_slice(),
        8 => rfft_8(data.try_into().unwrap()).as_mut_slice(),
        16 => rfft_16(data.try_into().unwrap()).as_mut_slice(),
        32 => rfft_32(data.try_into().unwrap()).as_mut_slice(),
        64 => rfft_64(data.try_into().unwrap()).as_mut_slice(),
        128 => rfft_128(data.try_into().unwrap()).as_mut_slice(),
        256 => rfft_256(data.try_into().unwrap()).as_mut_slice(),
        512 => rfft_512(data.try_into().unwrap()).as_mut_slice(),
        1024 => rfft_1024(data.try_into().unwrap()).as_mut_slice(),
        2048 => rfft_2048(data.try_into().unwrap()).as_mut_slice(),
        4096 => rfft_4096(data.try_into().unwrap()).as_mut_slice(),
        8192 => rfft_8192(data.try_into().unwrap()).as_mut_slice(),
        16384 => rfft_16384(data.try_into().unwrap()).as_mut_slice(),
        32768 => rfft_32768(data.try_into().unwrap()).as_mut_slice(),
        _ => panic!("invalid FFT length {}", data.len()),
    }
}

/// Move Nyquist frequency to its proper place from the imaginary part of DC (the first element).
/// The length of `data` must be 2 plus a power of two between 2 and 32768.
pub fn fix_nyquist(data: &mut [f32]) {
    let length = data.len();
    data[length - 2] = data[1];
    data[length - 1] = 0.0;
    data[1] = 0.0;
}

/// Fix negative frequencies in the power-of-two array to make the inverse FFT real-valued.
pub fn fix_negative(data: &mut [Complex32]) {
    let length = data.len();
    for i in length / 2 + 1..length {
        data[i] = data[length - i].conj();
    }
}

/// Perform inverse FFT in-place.
/// The length of `data` must be a power of two between 2 and 32768.
#[allow(unused_must_use)]
pub fn inverse_fft(data: &mut [Complex32]) {
    match data.len() {
        2 => {
            ifft_2(data.try_into().unwrap());
        }
        4 => {
            ifft_4(data.try_into().unwrap());
        }
        8 => {
            ifft_8(data.try_into().unwrap());
        }
        16 => {
            ifft_16(data.try_into().unwrap());
        }
        32 => {
            ifft_32(data.try_into().unwrap());
        }
        64 => {
            ifft_64(data.try_into().unwrap());
        }
        128 => {
            ifft_128(data.try_into().unwrap());
        }
        256 => {
            ifft_256(data.try_into().unwrap());
        }
        512 => {
            ifft_512(data.try_into().unwrap());
        }
        1024 => {
            ifft_1024(data.try_into().unwrap());
        }
        2048 => {
            ifft_2048(data.try_into().unwrap());
        }
        4096 => {
            ifft_4096(data.try_into().unwrap());
        }
        8192 => {
            ifft_8192(data.try_into().unwrap());
        }
        16384 => {
            ifft_16384(data.try_into().unwrap());
        }
        32768 => {
            ifft_32768(data.try_into().unwrap());
        }
        _ => panic!("invalid FFT length {}", data.len()),
    }
}
