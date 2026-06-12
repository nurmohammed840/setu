use rand::{
    distr::{SampleString, StandardUniform, uniform::SampleRange},
    prelude::*,
};

/// Generate a random string of the given length range.
pub fn rand_string<R, S>(len: S) -> impl FnOnce(&mut R) -> String
where
    R: Rng + ?Sized,
    S: SampleRange<usize>,
{
    move |r| {
        let len = r.random_range(len);
        StandardUniform.sample_string(r, len)
    }
}
