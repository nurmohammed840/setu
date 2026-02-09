# LIPI Specification

# Boolean

`Booleans` are encoded differently depending on context:

- **Field values (in a struct)**: Encoded directly in the **field header**. No additional bytes are used.
- **List values (in a `Vec<bool>`)**: The boolean values are packed into bytes, **8 values per byte**.

```
┌──────────────────┬───────────────────┐
| length (varint)  |  bytes (packed)   |
└──────────────────┴───────────────────┘
```

`length` is the number of boolean values, **NOT** the number of bytes in the packed representation.

Decoders **MUST** read the next `N` bytes as the packed boolean payload, where `N` is computed from `length`:

```rust
N = (length + 7) / 8
// Or
N = div_ceil(length, 8)
```

Since most programming languages perform integer division by truncation (floor division), `7` is added to `length` to ensure rounding up.  
Alternatively, a [ceiling division](https://doc.rust-lang.org/std/primitive.usize.html#method.div_ceil) can be performed; 

```rust
fn div_ceil(lhs: usize, rhs: usize) -> usize {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 { d + 1 } else { d }
}
```

### Packed Boolean Access

Booleans are stored **8 per byte**, in **LSB-first order**. To read or set a boolean at a given index `idx` (`0 ≤ idx < length`):

```rust
// Read the boolean at index
(bytes[idx / 8] & (1 << (idx % 8))) != 0;

// Set the boolean at index `idx` to true
bytes[idx / 8] |= 1 << (idx % 8);
```

# Integer

- `u8` and `i8` encoded as exactly **1 byte**.

- `f32` and `f64` encoded as IEEE-754 binary floating point in **little-endian** byte order:
  - `f32`: 32-bit (4 bytes)
  - `f64`: 64-bit (8 bytes)

- Unsigned integers `u16`, `u32`, `u64` encoded using **unsigned LEB128**.
- Signed integers `i16`, `i32`, `i64` encoded as:
  1. Apply the **ZigZag** transform to convert the signed integer into an unsigned integer.
  2. Encode the result using **unsigned LEB128**.

### LEB128 (VarInt)

[LEB128](https://en.wikipedia.org/wiki/LEB128) or Little Endian Base 128 is a variable-length code compression used to store arbitrarily large integers in a small number of bytes.

LEB128 encodes integers in groups of 7 bits, with 1 (MSB) bit indicating if the next byte follows.
Here is how the unsigned number `624485` gets encoded:

```
MSB ------------------ LSB
      10011000011101100101  In raw binary
     010011000011101100101  Padded to a multiple of 7 bits
 0100110  0001110  1100101  Split into 7-bit groups
00100110 10001110 11100101  Add high 1 bits on all but last (most significant) group to form bytes
    0x26     0x8E     0xE5  In hexadecimal

→ 0xE5 0x8E 0x26            Output stream (LSB to MSB)
```

### ZigZag Transform

`LEB128` is naturally efficient for **unsigned** integers because small values use fewer bytes.

However, if signed integers were encoded directly (two’s complement), small negative values like `-1` would appear as very large unsigned numbers (e.g. `0xFFFFFFFF`), which would take the maximum number of bytes in `LEB128`.

**ZigZag encoding** fixes this by mapping signed integers to unsigned integers so that values with small magnitude (both positive and negative) become small unsigned numbers:

- `0  -> 0`
- `-1 -> 1`
- `1  -> 2`
- `-2 -> 3`
- `2  -> 4`

This ensures that small negative numbers remain compact when encoded with unsigned `LEB128`.

ZigZag is computed using the following formula:

```rust
fn into_zig_zag(n: u64) -> i64 = (n << 1) ^ (n >> u64::BITS - 1)
fn from_zig_zag(n: u64) -> i64 = (n >>> 1) ^ - (n & 1)
```

# String

String is encoded as a length-prefixed sequence of UTF-8 bytes.

```
┌──────────────────┬───────────────────┐
| length (varint)  |  UTF-8 bytes      |
└──────────────────┴───────────────────┘
```

- The **length** is encoded as a **VarInt (LEB128)**.
- The utf-8 encoded bytes follows immediately after the length.


# Struct

In Lipi, data is encoded as a `Struct`, where each field carries the type information required to decode its value.

`Struct` is encoded as a **length-prefixed sequence of fields**, `length` is the number of fields in the struct.

```
┌──────────────────┬──────────────────┐
| length (varint)  |  Field, ...      |
└──────────────────┴──────────────────┘
```

Each **Field** is encoded as a **Header** followed by its **Value**.

```
┌──────────┬───────────┐
|  Header  |   Value   |
└──────────┴───────────┘
```

The field header contains the field **key** (as an integer) and a **4-bit type tag**. 
`Value` is decoded according to the type specified in the header.

The field **key** can be considered analogous to a JSON object’s field name.
This allows fields to be encoded in any order and enables forward and backward-compatible schema changes.

# Union

A `Union` is encoded like a `Struct`, but contains exactly one field.

```
┌──────────┬───────────┐
|  Header  |   Value   |
└──────────┴───────────┘
```

it is used to represent enums ([tagged union](https://en.wikipedia.org/wiki/Tagged_union)).

# Header

For numbers in the range `0..14`, the number is stored directly in the header and fits in a single byte.

```
7         3          0
┌─────────┬──────────┐
│   Num   │   Type   │
└─────────┴──────────┘

Type    → bits 0..4 (4 bits)
Number  → bits 4..8 (4 bits)
```


## Extended Header (Field Number ≥ 15)

If the number does not fit in 4 bits (`0..14`), the value `15` (`0b1111`) is stored in the `Num` field to indicate an extended header.

```
7          3           0
┌──────────┬───────────┬─────────────────────────────┐
│   1111   │   Type    │    Number (varint - 15)     |
└──────────┴───────────┘─────────────────────────────┘
0                      7                             N
```

The stored number is encoded as a VarInt and represents the value `(Number - 15)`

## Type

The field header contains a 4-bit `Type` tag (`0..=15`) which defines how the field value is encoded.

|    Tag    | Type                                     |
| :-------: | ---------------------------------------- |
|    `0`    | `false`                                  |
|    `1`    | `true`                                   |
|    `2`    | `u8`                                     |
|    `3`    | `i8`                                     |
|    `4`    | `f32`                                    |
|    `5`    | `f64`                                    |
|    `6`    | `UInt` (unsigned VarInt / ULEB128)       |
|    `7`    | `Int` (signed VarInt / ZigZag + ULEB128) |
|    `8`    | `String`                                 |
|    `9`    | `Struct`                                 |
|   `10`    | `Union`                                  |
|   `11`    | `List`                                   |
|   `12`    | `Table`                                  |
| `13..=15` | Reserved / other types                   |

**Note:** Boolean types `0` (`false`) and `1` (`true`) are encoded entirely in the header and have no value bytes.

### Optional Type

Lipi has no optional type. Optional values are represented by omitting the field from the `Struct`.

Optional values in a `List` are represented using a structured type.

### Reserved / Other Types

Type tags `13..=15` are **unused by Lipi**. Decoders **MUST** ignore unknown types and skip the next `N` bytes, where `N` is specified by the length prefix.

```
┌──────────────────┬───────────────────┐
| length (varint)  |  raw bytes        |
└──────────────────┴───────────────────┘
```

Decoders **MAY** use these type tags to determine how to decode the next `N` bytes,
but **MUST** read exactly `N` bytes before continuing with the next field.  

# List

A `List` in Lipi is encoded as a **length-prefixed** sequence of values,
`Header` encodes the **length** of the list and the value **type**.

```
┌──────────┬─────────────────┐
|  Header  |   Value, ...    |
└──────────┴─────────────────┘
```

# Table

```
┌───────────────────┬────────────────────┐
|  length (varint)  |     Column, ...    |
└───────────────────┴────────────────────┘
```

Column:

```
┌──────────┬─────────────────┐
|  Header  |   Value, ...    |
└──────────┴─────────────────┘
```

