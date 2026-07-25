enum Status {
  ok(0),
  cancelled(1),
  unknown(2),
  invalidArgument(3),
  deadlineExceeded(4),
  notFound(5),
  alreadyExists(6),
  permissionDenied(7),
  resourceExhausted(8),
  failedPrecondition(9),
  aborted(10),
  outOfRange(11),
  unimplemented(12),
  internal(13),
  unavailable(14),
  dataLoss(15);

  const Status(this.code);

  final int code;

  static Status from(int code) {
    final masked = code & 0xF;
    return Status.values.firstWhere((s) => s.code == masked);
  }

  @override
  String toString() => name;
}
