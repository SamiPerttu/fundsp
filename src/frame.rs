use super::*;
use typenum::*;

pub trait Frame: Copy + Clone
{
    type Sample: AudioFloat;
    type Channels: Unsigned;
    type Iterator: Iterator<Item = Self::Sample>;

    /// Creates Frame from an indexed function.
    fn from_fn(f: impl Fn(usize) -> Self::Sample) -> Self;

    /// Returns sample.
    fn channel(&self, i: usize) -> Self::Sample;

    /// Converts Frame into an iterator yielding the sample for each channel.
    fn channels(self) -> Self::Iterator;
}

/// An iterator that yields the sample for each channel in the frame by value.
#[derive(Clone)]
pub struct ChannelIter<F> {
    next_i: usize,
    frame: F,
}

macro_rules! impl_frame_for_fixed_size_array {
    ($($NChan:ident $N:expr, [$($idx:expr)*],)*) => {
        $(
            impl<F> Frame for [F; $N]
            where
                F: AudioFloat,
            {
                type Sample = F;
                type Channels = $NChan;
                type Iterator = ChannelIter<Self>;

                #[inline]
                fn channels(self) -> Self::Iterator { ChannelIter { next_i: 0, frame: self } }

                #[inline]
                fn channel(&self, i: usize) -> F { self[i] }

                #[inline]
                fn from_fn(_f: impl Fn(usize) -> Self::Sample) -> Self
                {
                    [$(_f($idx), )*]
                }
            }
        )*
    };
}

impl_frame_for_fixed_size_array! {
    U0  0,  [],
    U1  1,  [0],
    U2  2,  [0 1],
    U3  3,  [0 1 2],
    U4  4,  [0 1 2 3],
    U5  5,  [0 1 2 3 4],
    U6  6,  [0 1 2 3 4 5],
    U7  7,  [0 1 2 3 4 5 6],
    U8  8,  [0 1 2 3 4 5 6 7],
    U9  9,  [0 1 2 3 4 5 6 7 8],
    U10 10, [0 1 2 3 4 5 6 7 8 9],
    U11 11, [0 1 2 3 4 5 6 7 8 9 10],
    U12 12, [0 1 2 3 4 5 6 7 8 9 10 11],
    U13 13, [0 1 2 3 4 5 6 7 8 9 10 11 12],
    U14 14, [0 1 2 3 4 5 6 7 8 9 10 11 12 13],
    U15 15, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14],
    U16 16, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15],
    U17 17, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16],
    U18 18, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17],
    U19 19, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18],
    U20 20, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19],
    U21 21, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20],
    U22 22, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21],
    U23 23, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22],
    U24 24, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23],
    U25 25, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24],
    U26 26, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25],
    U27 27, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26],
    U28 28, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27],
    U29 29, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28],
    U30 30, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29],
    U31 31, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30],
    U32 32, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31],
}

macro_rules! impl_frame_for_sample {
    ($($T:ty)*) => {
        $(
            impl Frame for $T {
                type Sample = $T;
                type Channels = U1;
                type Iterator = ChannelIter<Self>;

                #[inline]
                fn channels(self) -> Self::Iterator { ChannelIter { next_i: 0, frame: self } }

                #[inline]
                fn channel(&self, i: usize) -> Self::Sample {
                    debug_assert!(i == 0);
                    *self
                }

                #[inline]
                fn from_fn(f: impl Fn(usize) -> Self::Sample) -> Self {
                    f(0)
                }
            }
        )*
    };
}

impl_frame_for_sample! {
    f32 f64
}

impl<F> Iterator for ChannelIter<F> where
    F: Frame,
{
    type Item = F::Sample;

    #[inline] fn next(&mut self) -> Option<Self::Item> {
        if self.next_i < F::Channels::USIZE {
            let item = self.frame.channel(self.next_i);
            self.next_i += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<F> ExactSizeIterator for ChannelIter<F>
where
    F: Frame,
{
    #[inline]
    fn len(&self) -> usize { F::Channels::USIZE - self.next_i }
}
