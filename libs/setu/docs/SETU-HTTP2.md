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
- `Content-Type: application/setu` or `application/setu+json`
- `rpc-id: 42`
- `rpc-timeout: 5000m` optional, If omitted, no timeout is applied.
- `rpc-encoding: ...` optional, If omitted, no compression is used.
- `rpc-accept-encoding: ...` optional

## RPC Request Body (Data Frame)

Length-Prefixed-Message:

```

```

# Responses

- `:status 200`
- `Content-Type: application/setu`
- `rpc-encoding: ...` optional
- `rpc-accept-encoding: ...` optional

### Reference

- [gRPC protocol over http/2](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md)