// ignore_for_file: constant_identifier_names

enum DataType {
  False(0),
  True(1),

  U8(2),
  I8(3),

  F32(4),
  F64(5),

  UInt(6),
  Int(7),

  Str(8),

  Struct(9),
  StructEnd(10),

  Union(11),
  List(12),
  Table(13),

  UnknownI(14),
  UnknownII(15);

  const DataType(this.code);
  final int code;

  static DataType fromCode(int code) {
    return DataType.values.firstWhere((ty) => ty.code == code);
  }

  static DataType fromBool(bool value) {
    return value ? DataType.True : DataType.False;
  }

  bool asBool() {
    switch (this) {
      case DataType.False:
        return false;
      case DataType.True:
        return true;
      default:
        throw ArgumentError('expected: False or True, found: $name');
    }
  }

  static DataType fromString(String str) {
    return DataType.values.firstWhere(
      (e) => e.name == str,
      orElse: () => throw TypeError(),
    );
  }

  void expected(DataType expected) {
    if (expected != this) {
      throw ArgumentError('expected: ${expected.name}, found: $name');
    }
  }
}
