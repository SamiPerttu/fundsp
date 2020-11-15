extern crate fundsp;

pub use fundsp::prelude::*;

// New components can be defined with the following return signature.
// Declaring the full arity in the signature enables use of the component
// in further combinations, as does the full type name.
// Signatures with generic number of channels can be challenging to write.
fn split_quad() -> Ac<impl AudioComponent<Inputs = U1, Outputs = U4>> {
    pass() & pass() & pass() & pass()
}
  
#[test]
fn test() {

    // Constants.
    let mut d = constant(1.0);
    assert!(d.inputs() == 0 && d.outputs() == 1);
    assert!(d.get_mono() == 1.0);
    let mut d = constant((2.0, 3.0));
    assert!(d.inputs() == 0 && d.outputs() == 2);
    assert!(d.get_stereo() == (2.0, 3.0));
    assert!(d.get_mono() == 2.0);
    let mut d = constant((4.0, 5.0, 6.0));
    assert!(d.inputs() == 0 && d.outputs() == 3);
    assert!(d.get_stereo() == (4.0, 5.0));

    assert!(split_quad().filter_mono(10.0) == 10.0);

    // Random stuff.
    let c = constant((2.0, 3.0)) * dc((2.0, 3.0));
    let e = c >> (pass() | pass());
    let mut f = e >> mul(0.5) + mul(0.5);
    assert!(f.inputs() == 0 && f.outputs() == 1);
    assert!(f.get_mono() == 6.5);

    // Test a visual cascade. The notation is slightly confusing.
    let c =     (pass()            | mul(1.0)          | add(1.0)           );
    let c = c / (pass()            | pass() * add(2.0)                      );
    let c = c / (mul(5.0)          | add(2.0)          | -add(1.0)          );
    let c = c / (mul(5.0)          + mul(5.0)          + pass()             );
    let mut c = c;
    let f = | x: f48, y: f48, z: f48 | 25.0 * x + 5.0 * y * z + 15.0 * y - z + 9.0;
    assert!(c.tick(&[0.0, 0.0, 0.0].into())[0] == f(0.0, 0.0, 0.0));
    assert!(c.tick(&[1.0, 0.0, 0.0].into())[0] == f(1.0, 0.0, 0.0));
    assert!(c.tick(&[0.0, 2.0, 0.0].into())[0] == f(0.0, 2.0, 0.0));
    assert!(c.tick(&[0.0, 0.0, 3.0].into())[0] == f(0.0, 0.0, 3.0));
    assert!(c.tick(&[2.0,-1.0, 2.0].into())[0] == f(2.0,-1.0, 2.0));
    assert!(c.tick(&[0.0, 3.0,-1.0].into())[0] == f(0.0, 3.0,-1.0));
}
