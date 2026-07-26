import 'dart:async';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:setu_client/errors.dart';
import 'package:setu_client/utils/bytes.dart';
import 'package:setu_client/utils/common.dart';

class HttpStream {
  HttpStream(Stream<Uint8List> stream, this._cancelToken)
    : _iterator = StreamIterator(stream);

  final StreamIterator<Uint8List> _iterator;
  final CancelToken _cancelToken;
  bool _eos = false;

  Future<Uint8List?> read() async {
    if (_eos) {
      throw EndOfData();
    }

    if (!await _iterator.moveNext()) {
      _eos = true;
      return null;
    }

    return _iterator.current;
  }

  Future<void> cancel() {
    // Abort the HTTP request.
    // Does it work, After server send response?
    _cancelToken.cancel();

    // Cancel the stream subscription.
    return _iterator.cancel();
  }
}

class StreamReader {
  StreamReader(this.stream);

  final HttpStream stream;
  var _data = Bytes.empty();

  Future<void> close() => stream.cancel();

  Future<Uint8List> readBytes(int len) async {
    if (len == 0) {
      return Uint8List(0);
    }

    final data = await read();
    if (len <= data.length) {
      return data.take(len);
    }

    final buf = BytesBuilder(copy: false);

    while (buf.length < len) {
      final data = await read();

      final remaining = len - buf.length;
      final takeN = math.min(remaining, data.length);
      buf.add(data.take(takeN));
    }

    return buf.takeBytes();
  }

  Future<int> readByte() async {
    final data = await read();
    return data.nextByte();
  }

  Future<Bytes> read() async {
    while (_data.isEmpty) {
      final bytes = expected(await stream.read(), "unexpected end of message");
      _data = Bytes(bytes);
    }
    return _data;
  }
}
