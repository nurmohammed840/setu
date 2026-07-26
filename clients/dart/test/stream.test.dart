import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:test/test.dart';

import 'package:setu_client/utils/stream.dart';

StreamReader createStream(List<List<int>> chunks) {
  Stream<Uint8List> source() async* {
    for (final chunk in chunks) {
      yield Uint8List.fromList(chunk);
    }
  }

  return StreamReader(HttpStream(source(), CancelToken()));
}

void main() {
  test('read byte', () async {
    final de = createStream([
      [],
      [4, 2],
      [],
      [42],
    ]);

    expect(await de.readByte(), 4);
    expect(await de.readByte(), 2);
    expect(await de.readByte(), 42);
  });

  test('read bytes', () async {
    final de = createStream([
      [],
      [1, 2, 3],
      [],
      [4],
      [],
      [5],
    ]);

    expect(await de.readBytes(2), [1, 2]);
    expect(await de.readBytes(0), isEmpty);
    expect(await de.readBytes(1), [3]);
    expect(await de.readBytes(0), isEmpty);
    expect(await de.readBytes(2), [4, 5]);

    expect(() => de.readBytes(1), throwsA(isA<Exception>()));
  });

  test('stream eof', () async {
    final de = createStream([
      [1],
      [2],
    ]);

    expect(() => de.readBytes(3), throwsA(isA<Exception>()));
  });


  test('partial read', () async {
    final de = createStream([
      [1, 2, 3, 4],
    ]);

    expect(await de.readByte(), 1);
    expect(await de.readBytes(2), [2, 3]);
    expect(await de.readByte(), 4);
  });
}
