use super::audionode::*;

/// Transform channels freely.
///
/// # Example
/// ```
/// # use fundsp::prelude::*;
/// # use fundsp::util::map;
/// let my_sum = map(|i: &Frame<f64, U2>| Frame::<f64, U1>::splat(i[0] + i[1]));
/// ```
// TODO: ConstantFrame (?) based version for prelude.
pub fn map<F, I, O>(f: F) -> Map<f64, F, I, O>
where
    F: Clone + FnMut(&Frame<f64, I>) -> Frame<f64, O>,
    I: Size<f64>,
    O: Size<f64>,
{
    Map::new(f)
}
