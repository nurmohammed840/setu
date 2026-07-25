import 'dart:typed_data';

import 'package:setu_client/utils/bytes.dart';
import 'package:test/test.dart';

void main() {
  test('take 2 bytes', () {
    final buf = Uint8List.fromList([1, 2, 3, 4, 5]);
    final (a, b) = takeBytes(2, buf);

    expect(a, [1, 2]);
    expect(b, [3, 4, 5]);
    expect(buf, [1, 2, 3, 4, 5]);
  });

  test('take 0 bytes', () {
    final (a, b) = takeBytes(0, Uint8List.fromList([1, 2, 3]));

    expect(a, isEmpty);
    expect(b, [1, 2, 3]);
  });

  test('take all bytes', () {
    final (a, b) = takeBytes(3, Uint8List.fromList([1, 2, 3]));

    expect(a, [1, 2, 3]);
    expect(b, isEmpty);
  });

  test('take more bytes', () {
    expect(() => takeBytes(5, Uint8List.fromList([1, 2, 3])), throwsRangeError);
  });

  test('take is cheap', () {
    final buf = Uint8List.fromList([1, 2, 3, 4]);
    final (a, b) = takeBytes(2, buf);

    buf[0] = 2;
    buf[3] = 3;

    expect(a, [2, 2]);
    expect(b, [3, 3]);
  });
}
