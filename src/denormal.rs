//! Denormal prevention.

/// Attempt to set processor flags to prevent denormals.
#[inline]
pub fn prevent_denormals() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
        #[allow(deprecated)]
        use core::arch::x86_64::_mm_setcsr;

        #[cfg(all(target_arch = "x86", target_feature = "sse"))]
        use core::arch::x86::_mm_setcsr;

        // Treat denormals as zero while enabling all interrupt masks.
        #[allow(deprecated)]
        unsafe {
            _mm_setcsr(0x9fc0)
        };
    }
}
