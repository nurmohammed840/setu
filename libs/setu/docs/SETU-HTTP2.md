# Introduction

Setu uses HTTP/2 by default, but it can work over any transport that supports:

- Multiplexing
- Flow control
- Connection management
- Streaming
- TLS
  
When using raw Socket (ex: TCP) or protocols such as WebSocket, these capabilities must be implemented by the application layer.

# RPC Request

- `:method POST`
- `:path /ServiceName`
- `:scheme https`
- `:authority localhost:50050` optional
- `content-type: application/setu` or `application/setu+json`
- `rpc-id: 42`
- `rpc-timeout: 5000m` optional, If omitted, no timeout is applied.
- `rpc-encoding: ...` optional, If omitted, no compression is used.
- `rpc-accept-encoding: ...` optional

## RPC Request Body (Data Frame)

Length-Prefixed-Message:

```rs
struct Frame {
    // 1 byte
    header: FrameHeader,
    // 1-4 bytes
    length: u32,
    // N bytes
    payload: [u8]
}

struct FrameHeader {
    // 1 bit
    is_compressed: bool,
    // 1 bit
    is_trailer: bool,
    // 2 bit
    len_size: u8,
    // 4 bit 
    code: u8, // Must be `0` if `!is_trailer`
}

fn parse_header(byte: u8) -> FrameHeader {
    FrameHeader {
        is_compressed: (byte & 0b1) == 0b1,
        is_trailer: (byte & 0b10) == 0b10,
        len_size: ((byte >> 2) & 0b11) + 1,
        code: byte >> 4,
    }
}

fn parse_length_big_endian(size: u8) -> u32 {
    let length = 0;
    for _ in 0..size {
        length = (length << 8) | next_byte();
    }
    return length;
}
```

# Responses

- `:status 200`
- `content-type: application/setu`
- `rpc-encoding: ...` optional
- `rpc-accept-encoding: ...` optional

### Reference

- [gRPC protocol over http/2](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md)