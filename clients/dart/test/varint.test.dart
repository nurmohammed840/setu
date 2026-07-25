import 'package:setu_client/lipi/zigzag.dart';
import 'package:test/test.dart';

void main() {
  group('ZigZag Encoding & Decoding', () {
    void checkZigZag(BigInt actual, BigInt encoded) {
      expect(zigzagEncode(actual), equals(encoded), reason: 'Failed encoding for $actual');
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
        BigInt.parse('9223372036854775807'),  // i64::MAX
        BigInt.parse('18446744073709551614'), // u64::MAX - 1
      );
    });

    test('handles 32-bit bounds (i32::MIN and i32::MAX)', () {
      checkZigZag(
        BigInt.parse('-2147483648'),          // i32::MIN
        BigInt.parse('4294967295'),           // u32::MAX
      );
      checkZigZag(
        BigInt.parse('2147483647'),           // i32::MAX
        BigInt.parse('4294967294'),           // u32::MAX - 1
      );
    });
  });
}