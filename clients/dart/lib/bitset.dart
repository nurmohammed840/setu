import 'dart:typed_data';

abstract interface class BitSetRead {
  int capacity();
  bool isEmpty();
  bool has(int index);
  bool? get(int index);
}

abstract interface class BitSetWrite {
  void clear();
  bool set(int index);
  bool? remove(int index);
}

class BitVec implements BitSetRead, BitSetWrite {
  final Uint8List _bytes;

  BitVec.fromLength(int len) : _bytes = Uint8List(boolPackedLen(len));
  BitVec.fromBytes(Uint8List bytes) : _bytes = bytes;

  Uint8List asBytes() => _bytes;

  @override
  int capacity() => _bytes.length * 8;

  @override
  bool isEmpty() => _bytes.every((slot) => slot == 0);

  @override
  bool has(int index) => get(index) ?? false;

  @override
  bool? get(int index) {
    final slotIdx = index ~/ 8;
    final mask = 1 << (index % 8);

    if (slotIdx >= _bytes.length) {
      return null;
    }

    return (_bytes[slotIdx] & mask) != 0;
  }

  @override
  void clear() {
    _bytes.fillRange(0, _bytes.length, 0);
  }

  @override
  bool set(int index) {
    final slotIdx = index ~/ 8;
    final mask = 1 << (index % 8);

    if (slotIdx >= _bytes.length) {
      throw RangeError('Out of bounds slot index: $slotIdx');
    }

    final oldValue = (_bytes[slotIdx] & mask) != 0;
    _bytes[slotIdx] |= mask;

    return oldValue;
  }

  @override
  bool? remove(int index) {
    final slotIdx = index ~/ 8;
    final mask = 1 << (index % 8);

    if (slotIdx >= _bytes.length) {
      return null;
    }

    final oldValue = (_bytes[slotIdx] & mask) != 0;
    _bytes[slotIdx] &= ~mask;

    return oldValue;
  }
}

int boolPackedLen(int len) {
  if (len < 0) {
    throw RangeError('length $len cannot be negative');
  }
  return (len + 7) ~/ 8;
}

BitVec bitvecFrom(List<bool> bools) {
  final bv = BitVec.fromLength(bools.length);
  for (var i = 0; i < bools.length; i++) {
    if (bools[i]) {
      bv.set(i);
    }
  }
  return bv;
}

List<bool> bitvecToBools(BitVec bv, int len) {
  return List<bool>.generate(len, (i) => bv.has(i));
}
