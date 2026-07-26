class EndOfData implements Exception {
  final String message;

  const EndOfData([this.message = 'read after end of stream']);

  @override
  String toString() => 'EndOfData: $message';
}

class ProtocolError implements Exception {
  final String? message;

  const ProtocolError([this.message]);

  @override
  String toString() =>
      message == null ? 'ProtocolError' : 'ProtocolError: $message';
}
