import 'dart:typed_data';

T expected<T>(T? data, [String message = 'expected value']) {
  if (data == null) {
    throw TypeError();
  }
  return data;
}

void assertExpr(
  bool expr, {
  Object Function(String message)? error,
  String Function()? message,
}) {
  if (expr) return;

  final msg = message?.call() ?? '';

  if (error != null) {
    throw error(msg);
  }

  throw Exception(msg);
}

int checkOverflowInt(int num, int bits) {
  final max = (1 << (bits - 1)) - 1;
  final min = -(1 << (bits - 1));

  if (num < min || num > max) {
    throw RangeError('Int$bits overflow: $num');
  }

  return num;
}

int checkOverflowUint(int num, int bits) {
  final max = (1 << bits) - 1;

  if (num < 0 || num > max) {
    throw RangeError('Uint$bits overflow: $num');
  }

  return num;
}

bool isLittleEndian() {
  final bytes = Uint8List(4);
  final data = ByteData.sublistView(bytes);

  data.setUint32(0, 0x11223344, Endian.host);

  return bytes[0] == 0x44;
}

final bool isLittleEndianHost = isLittleEndian();