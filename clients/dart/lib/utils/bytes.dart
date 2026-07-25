import 'dart:typed_data';

import 'package:setu_client/utils/common.dart';

(Uint8List, Uint8List) takeBytes(int n, Uint8List buf) {
  assertExpr(
    n <= buf.length,
    error: RangeError.new,
    message: () => 'takeBytes($n) exceeds buffer length ${buf.length}',
  );
  return (Uint8List.sublistView(buf, 0, n), Uint8List.sublistView(buf, n));
}

class Bytes {
  factory Bytes.empty() {
    return Bytes(Uint8List(0));
  }

  Bytes(this._data);

  Uint8List _data;

  int get length => _data.length;
  bool get isEmpty => _data.isEmpty;
  Uint8List get remaining => _data;

  int nextByte() {
    final (bytes, ptr) = takeBytes(1, _data);
    _data = ptr;
    return bytes[0];
  }

  Uint8List take(int len) {
    final (bytes, ptr) = takeBytes(len, _data);
    _data = ptr;
    return bytes;
  }
}
