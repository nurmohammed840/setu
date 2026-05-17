import { assertEquals, assertThrows } from "../dev-deps.ts";
import { Timeout } from "../src/timeout.ts";

function check(unit: string, timeout: Timeout) {
    let s = timeout.toString();
    assertEquals(Timeout.fromString(s), timeout);
    assertEquals(s, unit);
}

Deno.test("valid_cases", () => {
    check("1H", Timeout.hour(1));
    check("2M", Timeout.minute(2));
    check("3S", Timeout.second(3));
    check("10m", Timeout.millisecond(10));
});

Deno.test("invalid_cases", () => {
    assertThrows(() => Timeout.fromString(" 1H "));
    assertThrows(() => Timeout.fromString("1"));
    assertThrows(() => Timeout.fromString("S"));
    assertThrows(() => Timeout.fromString("5X"));
    assertThrows(() => Timeout.fromString("aS"));
});