use rand::{
    distr::{SampleString, StandardUniform, uniform::SampleRange},
    prelude::*,
};
use std::cell::Cell;

/// Generate a random string of the given length range.
pub fn sample_string<R>(len: impl SampleRange<usize>) -> impl FnOnce(&mut R) -> String
where
    R: Rng + ?Sized,
{
    move |r| {
        let len = r.random_range(len);
        StandardUniform.sample_string(r, len)
    }
}

pub fn _default_sample<R, T>(_: &mut R) -> T
where
    R: Rng + ?Sized,
    T: Default,
{
    T::default()
}

thread_local! {
    static DEPTH: Cell<u8> = const { Cell::new(0) };
}

fn with_depth<T>(max_depth: u8, f: impl FnOnce() -> T) -> T
where
    T: Default,
{
    DEPTH.with(|d| {
        let depth = d.get();

        if depth >= max_depth {
            return T::default();
        }

        if depth == u8::MAX {
            panic!("maximum recursion depth exceeded (u8 overflow)");
        }

        d.set(depth + 1);
        let value = f();
        d.set(depth); // decrement depth
        value
    })
}

pub fn sample_list<R, C, T>(max_depth: u8, len: impl SampleRange<usize>) -> impl FnOnce(&mut R) -> C
where
    R: Rng + ?Sized,
    C: FromIterator<T> + Default,
    StandardUniform: Distribution<T>,
{
    move |r| {
        with_depth(max_depth, || {
            let len = r.random_range(len);
            (0..len).map(|_| r.random()).collect()
        })
    }
}

pub fn sample_map<R, C, T>(max_depth: u8, len: impl SampleRange<usize>) -> impl FnOnce(&mut R) -> C
where
    R: Rng + ?Sized,
    C: FromIterator<(String, T)> + Default,
    StandardUniform: Distribution<T>,
{
    move |r| {
        with_depth(max_depth, || {
            let len = r.random_range(len);
            (0..len)
                .map(|_| (sample_string(0..5)(r), r.random()))
                .collect()
        })
    }
}
