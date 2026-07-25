import 'package:setu_client/bitset.dart';
import 'package:test/test.dart';

void main() {
  test('create_bit_set', () {
    final bs1 = bitvecFrom([true, false]);
    final bs2 = bitvecFrom([true, false]);

    expect(bs1.asBytes(), equals(bs2.asBytes()));

    final bs3 = BitVec.fromLength(2);
    bs3.set(0);

    expect(bs1.asBytes(), equals(bs3.asBytes()));
  });

  test('get_behavior', () {
    final bs = BitVec.fromLength(8);

    expect(bs.get(8), isNull);

    bs
      ..set(0)
      ..set(7);

    expect(bs.get(0), isTrue);
    expect(bs.get(7), isTrue);

    expect(bs.get(999), isNull);

    expect(bs.isEmpty(), isFalse);

    bs.clear();

    expect(bs.isEmpty(), isTrue);
  });

  test('has_and_is_lsb_first', () {
    final bs = BitVec.fromLength(8);

    bs.set(0);
    expect(bs.asBytes(), equals([0x01]));

    bs.set(1);
    expect(bs.asBytes(), equals([0x03]));

    bs.set(7);
    expect(bs.asBytes(), equals([0x83]));

    expect(bs.has(0), isTrue);
    expect(bs.has(1), isTrue);
    expect(bs.has(7), isTrue);

    expect(bs.has(2), isFalse);
  });

  test('insert_and_returns_old_value', () {
    final bs = BitVec.fromLength(16);

    expect(bs.set(3), isFalse);
    expect(bs.has(3), isTrue);

    expect(bs.set(3), isTrue);
    expect(bs.has(3), isTrue);
  });

  test('remove_and_returns_old_value', () {
    final bs = BitVec.fromLength(16);

    expect(bs.remove(5), isFalse);
    expect(bs.has(5), isFalse);

    expect(bs.set(5), isFalse);
    expect(bs.has(5), isTrue);

    expect(bs.remove(5), isTrue);
    expect(bs.has(5), isFalse);
  });

  test('out_of_bounds_insert_and_remove', () {
    final bs = BitVec.fromLength(8);

    expect(() => bs.set(8), throwsRangeError);

    expect(bs.remove(8), isNull);

    expect(bs.has(8), isFalse);
    expect(bs.has(9999), isFalse);
  });

  test('bools_roundtrip', () {
    final input = <bool>[
      true, false, true, true, false, false, false, true,
      false, true, false, false, true, false, true, false,
      true,
    ];

    final bs = bitvecFrom(input);
    final output = bitvecToBools(bs, input.length);

    expect(output, equals(input));
  });

  test('cross_byte_boundaries', () {
    final bs = BitVec.fromLength(16);

    bs
      ..set(7)
      ..set(8)
      ..set(15);

    expect(bs.has(7), isTrue);
    expect(bs.has(8), isTrue);
    expect(bs.has(15), isTrue);

    expect(bs.asBytes(), equals([0x80, 0x81]));
  });
}