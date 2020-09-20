#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(trait_alias)]

pub mod filter;
pub mod sample;
pub mod audiounit;
pub mod audiocomponent;
pub mod prelude;
pub mod lti;
pub mod noise;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

