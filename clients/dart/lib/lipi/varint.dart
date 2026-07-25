import 'dart:typed_data';

import 'package:setu_client/utils/bytes.dart';
import 'package:setu_client/utils/common.dart';

Uint8List encodeVarInt(BigInt num) {
  assertExpr(
    num >= BigInt.zero,
    error: RangeError.new,
    message: () => 'expected unsigned number: found $num',
  );

  final buf = <int>[];

  while (num > BigInt.from(0x7F)) {
    buf.add(((num & BigInt.from(0xFF)).toInt()) | 0x80);
    num = num >> 7;
  }

  buf.add(num.toInt());

  return Uint8List.fromList(buf);
}

BigInt decodeVarInt(Bytes bytes) {
  var result = BigInt.zero;
  var shift = 0;

  while (true) {
    final byte = BigInt.from(bytes.nextByte());

    if (shift == 63 && byte >= BigInt.two) {
      throw Exception('invalid variable-length integer');
    }

    if ((byte & BigInt.from(0x80)) == BigInt.zero) {
      return result | (byte << shift);
    }

    result |= (byte & BigInt.from(0x7F)) << shift;
    shift += 7;
  }
}
