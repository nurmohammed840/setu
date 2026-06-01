import { assert } from "./utils/common.ts";

export enum TimeoutUnit {
    Hour = "H",
    Minute = "M",
    Second = "S",
    Millisecond = "m",
}

export class Timeout {
    private constructor(
        public readonly unit: TimeoutUnit,
        public readonly value: number,
    ) { }

    static hour(hours: number): Timeout {
        return new Timeout(TimeoutUnit.Hour, hours);
    }

    static minute(mins: number): Timeout {
        return new Timeout(TimeoutUnit.Minute, mins);
    }

    static second(secs: number): Timeout {
        return new Timeout(TimeoutUnit.Second, secs);
    }

    static millisecond(ms: number): Timeout {
        return new Timeout(TimeoutUnit.Millisecond, ms);
    }

    toString(): string {
        return `${this.value}${this.unit}`;
    }

    duration(): number {
        switch (this.unit) {
            case TimeoutUnit.Hour:
                return this.value * 60 * 60 * 1000;

            case TimeoutUnit.Minute:
                return this.value * 60 * 1000;

            case TimeoutUnit.Second:
                return this.value * 1000;

            case TimeoutUnit.Millisecond:
                return this.value;
        }
    }

    static fromString(input: string): Timeout {
        assert(input.length >= 2, SyntaxError, "timeout: invalid format");

        const numPart = input.slice(0, -1);
        const unit = input.slice(-1);

        const value = Number.parseInt(numPart, 10);

        assert(Number.isFinite(value), TypeError, "timeout: invalid number");

        switch (unit) {
            case "H":
                return Timeout.hour(value);

            case "M":
                return Timeout.minute(value);

            case "S":
                return Timeout.second(value);

            case "m":
                return Timeout.millisecond(value);

            default:
                throw new Error("timeout: unknown unit");
        }
    }

    equals(other: Timeout): boolean {
        console.log({ other });
        return this.unit === other.unit && this.value === other.value;
    }
}
