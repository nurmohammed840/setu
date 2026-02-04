// mod structure;

// use crate::structure::Types;
// use cor::{Decoder, Entries};
// use criterion::{Criterion, criterion_group, criterion_main};

// fn bench(c: &mut Criterion, name: &str, iter: impl FnMut() + Copy) {
//     c.bench_function(name, |b| b.iter(iter));
// }

// fn benchmark(c: &mut Criterion) {
//     let types = Types::new();
//     let raw = types.to_bytes();
//     let entries = Entries::parse(&mut &raw[..]).unwrap();
//     assert_eq!(types, Types::decode(&entries).unwrap());

//     bench(c, "encode", || assert!(!types.to_bytes().is_empty()));
//     bench(c, "parse", || {
//         assert!(!Entries::parse(&mut &raw[..]).unwrap().is_empty());
//     });
//     bench(c, "decode", || assert!(Types::decode(&entries).is_ok()));
//     bench(c, "parse and decode", || {
//         assert!(Types::from_bytes(&raw[..]).is_ok());
//     });
// }

// criterion_group!(benches, benchmark);
// criterion_main!(benches);
