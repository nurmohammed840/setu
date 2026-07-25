import 'package:setu_client/utils/common.dart';

enum TimeoutUnit {
  hour('H'),
  minute('M'),
  second('S'),
  millisecond('m');

  const TimeoutUnit(this.symbol);

  final String symbol;
}

class Timeout {
  factory Timeout.fromString(String input) {
    assertExpr(
      input.length >= 2,
      error: FormatException.new,
      message: () => 'timeout: invalid format',
    );

    final numPart = input.substring(0, input.length - 1);
    final unit = input.substring(input.length - 1);

    final value = int.tryParse(numPart);

    if (value == null) {
      throw FormatException('timeout: invalid number');
    }

    return switch (unit) {
      'H' => Timeout.hour(value),
      'M' => Timeout.minute(value),
      'S' => Timeout.second(value),
      'm' => Timeout.millisecond(value),
      _ => throw FormatException('timeout: unknown unit'),
    };
  }

  const Timeout._(this.unit, this.value);

  final TimeoutUnit unit;
  final int value;

  const Timeout.hour(int hours) : this._(TimeoutUnit.hour, hours);
  const Timeout.minute(int minutes) : this._(TimeoutUnit.minute, minutes);
  const Timeout.second(int seconds) : this._(TimeoutUnit.second, seconds);
  const Timeout.millisecond(int milliseconds)
    : this._(TimeoutUnit.millisecond, milliseconds);

  @override
  String toString() => '$value${unit.symbol}';

  Duration get duration => switch (unit) {
    TimeoutUnit.hour => Duration(hours: value),
    TimeoutUnit.minute => Duration(minutes: value),
    TimeoutUnit.second => Duration(seconds: value),
    TimeoutUnit.millisecond => Duration(milliseconds: value),
  };

  @override
  bool operator ==(Object other) =>
      other is Timeout && other.unit == unit && other.value == value;

  @override
  int get hashCode => Object.hash(unit, value);
}
