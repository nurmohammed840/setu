//! Run benchmark:
//!
//! ```
//! cd libs\lipi\
//! cargo bench --bench basic
//! ```

mod data {
    include!("../tests/data.rs");
}

use crate::data::Types;
use criterion::{Criterion, criterion_group, criterion_main};
use lipi::*;

type Data = (Types, Types, Types);

fn run_bench(c: &mut Criterion, name: &str, iter: impl FnMut() + Copy) {
    c.bench_function(name, |b| b.iter(iter));
}

fn benchmark(c: &mut Criterion) {
    let data = (Types::min(), Types::mid(), Types::max());

    let raw = data.to_bytes().unwrap();
    assert_eq!(data, Data::decode(&mut &raw[..]).unwrap());

    run_bench(c, "encode", || {
        assert_eq!(data.to_bytes().unwrap().len(), raw.len());
    });

    run_bench(c, "decode", || {
        let new_data = Data::decode(&mut &raw[..]).unwrap();
        assert_eq!(new_data.2.bool, data.2.bool);
    });

    run_bench(c, "encode and decode", || {
        let raw = data.to_bytes().unwrap();
        let new_data = Data::decode(&mut &raw[..]).unwrap();
        assert_eq!(new_data.2.bool, data.2.bool);
    });
}

criterion_group!(benches, benchmark);

criterion_main!(benches);
