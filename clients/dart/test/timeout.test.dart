import 'package:test/test.dart' hide Timeout;
import 'package:setu_client/timeout.dart';

void main() {
  void check(String text, Timeout timeout) {
    expect(Timeout.fromString(text), equals(timeout));
    expect(timeout.toString(), equals(text));
  }

  test('valid cases', () {
    check('1H', const Timeout.hour(1));
    check('2M', const Timeout.minute(2));
    check('3S', const Timeout.second(3));
    check('10m', const Timeout.millisecond(10));
  });

  test('invalid cases', () {
    expect(() => Timeout.fromString(' 1H '), throwsA(isA<FormatException>()));
    expect(() => Timeout.fromString('1'), throwsA(isA<FormatException>()));
    expect(() => Timeout.fromString('S'), throwsA(isA<FormatException>()));
    expect(() => Timeout.fromString('5X'), throwsA(isA<FormatException>()));
    expect(() => Timeout.fromString('aS'), throwsA(isA<FormatException>()));
  });
}
