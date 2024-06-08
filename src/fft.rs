//! Interface to microfft.
use core::array::from_fn;
use microfft::inverse::*;
use microfft::real::*;
use num_complex::Complex32;

/// Perform real-valued FFT. The length of `input` must be a power of two between 2 and 32768.
/// The length of `output` must be half that of `input` plus one.
pub fn real_fft(input: &[f32], output: &mut [Complex32]) {
    assert!(input.len() == (output.len() - 1) * 2);
    match input.len() {
        2 => {
            let mut tmp: [f32; 2] = from_fn(|i| input[i]);
            output[..1].copy_from_slice(rfft_2(&mut tmp));
        }
        4 => {
            let mut tmp: [f32; 4] = from_fn(|i| input[i]);
            output[..2].copy_from_slice(rfft_4(&mut tmp));
        }
        8 => {
            let mut tmp: [f32; 8] = from_fn(|i| input[i]);
            output[..4].copy_from_slice(rfft_8(&mut tmp));
        }
        16 => {
            let mut tmp: [f32; 16] = from_fn(|i| input[i]);
            output[..8].copy_from_slice(rfft_16(&mut tmp));
        }
        32 => {
            let mut tmp: [f32; 32] = from_fn(|i| input[i]);
            output[..16].copy_from_slice(rfft_32(&mut tmp));
        }
        64 => {
            let mut tmp: [f32; 64] = from_fn(|i| input[i]);
            output[..32].copy_from_slice(rfft_64(&mut tmp));
        }
        128 => {
            let mut tmp: [f32; 128] = from_fn(|i| input[i]);
            output[..64].copy_from_slice(rfft_128(&mut tmp));
        }
        256 => {
            let mut tmp: [f32; 256] = from_fn(|i| input[i]);
            output[..128].copy_from_slice(rfft_256(&mut tmp));
        }
        512 => {
            let mut tmp: [f32; 512] = from_fn(|i| input[i]);
            output[..256].copy_from_slice(rfft_512(&mut tmp));
        }
        1024 => {
            let mut tmp: [f32; 1024] = from_fn(|i| input[i]);
            output[..512].copy_from_slice(rfft_1024(&mut tmp));
        }
        2048 => {
            let mut tmp: [f32; 2048] = from_fn(|i| input[i]);
            output[..1024].copy_from_slice(rfft_2048(&mut tmp));
        }
        4096 => {
            let mut tmp: [f32; 4096] = from_fn(|i| input[i]);
            output[..2048].copy_from_slice(rfft_4096(&mut tmp));
        }
        8192 => {
            let mut tmp: [f32; 8192] = from_fn(|i| input[i]);
            output[..4096].copy_from_slice(rfft_8192(&mut tmp));
        }
        16384 => {
            let mut tmp: [f32; 16384] = from_fn(|i| input[i]);
            output[..8192].copy_from_slice(rfft_16384(&mut tmp));
        }
        32768 => {
            let mut tmp: [f32; 32768] = from_fn(|i| input[i]);
            output[..16384].copy_from_slice(rfft_32768(&mut tmp));
        }
        _ => panic!("Unsupported FFT length."),
    }
    output[output.len() - 1] = Complex32::new(output[0].im, 0.0);
    output[0] = Complex32::new(output[0].re, 0.0);
}

/// Perform inverse FFT. The length of `input` must be a power of two between 2 and 32768.
/// The length of `output` must be equal to that of `input`.
pub fn inverse_fft(input: &[Complex32], output: &mut [Complex32]) {
    assert!(input.len() == output.len());
    match input.len() {
        2 => {
            let mut tmp: [Complex32; 2] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_2(&mut tmp));
        }
        4 => {
            let mut tmp: [Complex32; 4] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_4(&mut tmp));
        }
        8 => {
            let mut tmp: [Complex32; 8] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_8(&mut tmp));
        }
        16 => {
            let mut tmp: [Complex32; 16] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_16(&mut tmp));
        }
        32 => {
            let mut tmp: [Complex32; 32] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_32(&mut tmp));
        }
        64 => {
            let mut tmp: [Complex32; 64] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_64(&mut tmp));
        }
        128 => {
            let mut tmp: [Complex32; 128] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_128(&mut tmp));
        }
        256 => {
            let mut tmp: [Complex32; 256] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_256(&mut tmp));
        }
        512 => {
            let mut tmp: [Complex32; 512] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_512(&mut tmp));
        }
        1024 => {
            let mut tmp: [Complex32; 1024] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_1024(&mut tmp));
        }
        2048 => {
            let mut tmp: [Complex32; 2048] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_2048(&mut tmp));
        }
        4096 => {
            let mut tmp: [Complex32; 4096] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_4096(&mut tmp));
        }
        8192 => {
            let mut tmp: [Complex32; 8192] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_8192(&mut tmp));
        }
        16384 => {
            let mut tmp: [Complex32; 16384] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_16384(&mut tmp));
        }
        32768 => {
            let mut tmp: [Complex32; 32768] = from_fn(|i| input[i]);
            output.copy_from_slice(ifft_32768(&mut tmp));
        }
        _ => panic!("Unsupported FFT length."),
    }
}
