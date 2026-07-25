import 'dart:typed_data';

import 'package:setu_client/utils/bytes.dart';
import 'package:test/test.dart';

import 'package:setu_client/lipi/varint.dart';
import 'package:setu_client/lipi/zigzag.dart';

int encodedVarIntLen(BigInt value) {
  final bits = (value | BigInt.one).bitLength;
  return (bits + 6) ~/ 7;
}

BigInt decodeVarIntBytes(List<int> bytes) {
  return decodeVarInt(Bytes(Uint8List.fromList(bytes)));
}

List<int> encodeVarIntBytes(BigInt value) {
  return encodeVarInt(value);
}

BigInt maxNum(int bits) => (BigInt.one << bits) - BigInt.one;

final u64Max = BigInt.parse('0xFFFFFFFFFFFFFFFF');
final i64Max = BigInt.parse('0x7FFFFFFFFFFFFFFF');
const u32Max = 0xffffffff;
const i32Max = 0x7fffffff;

void main() {
  test("VarInt Encoding", () {
    void check(BigInt value) {
      final encoded = encodeVarIntBytes(value);

      expect(encoded.length, encodedVarIntLen(value));
      expect(decodeVarIntBytes(encoded), value);
    }

    for (var i = 0; i < 1000; i++) {
      check(BigInt.from(i));
    }

    // Powers of 2
    for (int exp = 0; ; exp++) {
      final n = BigInt.from(2).pow(exp);
      if (n > maxNum(64)) break;
      check(n);
    }

    // Powers of 3
    for (int exp = 0; ; exp++) {
      final n = BigInt.from(3).pow(exp);
      if (n > maxNum(64)) break;
      check(n);
    }

    check(u64Max);
    check(u64Max - BigInt.one);

    check(i64Max);
    check(i64Max + BigInt.one);
    check(i64Max - BigInt.one);

    check(BigInt.from(u32Max));
    check(BigInt.from(u32Max + 1));
    check(BigInt.from(u32Max - 1));

    check(BigInt.from(i32Max));
    check(BigInt.from(i32Max + 1));
    check(BigInt.from(i32Max - 1));
  });

  test('VarInt Decoding', () {
    void check(BigInt value, List<int> encoded) {
      expect(encodeVarIntBytes(value), encoded);

      expect(encoded.length, encodedVarIntLen(value));
      expect(decodeVarIntBytes(encoded), value);
    }

    check(BigInt.one, [0x01]);
    check(BigInt.from(127), [0x7F]);
    check(BigInt.from(128), [0x80, 0x01]);
    check(BigInt.from(300), [0xAC, 0x02]);

    for (var exp = 14; exp <= 63; exp += 7) {
      final groups = exp ~/ 7;

      check((BigInt.one << exp) - BigInt.one, [
        ...List.filled(groups - 1, 0xFF),
        0x7F,
      ]);

      check(BigInt.one << exp, [...List.filled(groups, 0x80), 0x01]);
    }

    check(BigInt.zero, [0x00]);
    check(u64Max, const [
      0xff,
      0xff,
      0xff,
      0xff,
      0xff,
      0xff,
      0xff,
      0xff,
      0xff,
      0x01,
    ]);
    check(BigInt.from(0xffffffff), const [0xff, 0xff, 0xff, 0xff, 0x0f]);
    expect(
      () => decodeVarIntBytes(const [
        0xff,
        0xff,
        0xff,
        0xff,
        0xff,
        0xff,
        0xff,
        0xff,
        0xff,
        0x02,
      ]),
      throwsException,
    );
  });

  group('ZigZag Encoding & Decoding', () {
    void checkZigZag(BigInt actual, BigInt encoded) {
      expect(
        zigzagEncode(actual),
        equals(encoded),
        reason: 'Failed encoding for $actual',
      );
      expect(
        BigInt.from(zigzagDecode(encoded)),
        equals(actual),
        reason: 'Failed decoding for $encoded',
      );
    }

    test('encodes and decodes basic values correctly', () {
      checkZigZag(BigInt.zero, BigInt.zero);
      checkZigZag(BigInt.from(-1), BigInt.from(1));
      checkZigZag(BigInt.from(1), BigInt.from(2));
      checkZigZag(BigInt.from(-2), BigInt.from(3));
      checkZigZag(BigInt.from(2), BigInt.from(4));
    });

    test('handles 64-bit bounds (i64::MIN and i64::MAX)', () {
      checkZigZag(
        BigInt.parse('-9223372036854775808'), // i64::MIN
        BigInt.parse('18446744073709551615'), // u64::MAX
      );
      checkZigZag(
        BigInt.parse('9223372036854775807'), // i64::MAX
        BigInt.parse('18446744073709551614'), // u64::MAX - 1
      );
    });

    test('handles 32-bit bounds (i32::MIN and i32::MAX)', () {
      checkZigZag(
        BigInt.parse('-2147483648'), // i32::MIN
        BigInt.parse('4294967295'), // u32::MAX
      );
      checkZigZag(
        BigInt.parse('2147483647'), // i32::MAX
        BigInt.parse('4294967294'), // u32::MAX - 1
      );
    });
  });
}
