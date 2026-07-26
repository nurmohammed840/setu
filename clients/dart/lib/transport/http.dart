import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:setu_client/http/client.dart';
import 'package:setu_client/setu/http_headers.dart';
import 'package:setu_client/timeout.dart';

class RPC {
  static var uri = Uri.parse('https://localhost:443/');
  static var timeout = Timeout.minute(2);
  static var dio = createHttpClient();

  static Future<ResponseBody> call({
    required int id,
    required Uint8List body,

    Dio? dio,
    Uri? uri,
    Timeout? timeout,
    CancelToken? cancelToken,
  }) async {
    final rpcUri = (uri ?? RPC.uri).toString();
    final rpcTimeout = timeout ?? RPC.timeout;

    final response = await (dio ?? RPC.dio).post<ResponseBody>(
      rpcUri,
      data: Stream.fromIterable([body]),
      cancelToken: cancelToken,
      options: Options(
        responseType: ResponseType.stream,
        sendTimeout: rpcTimeout.duration,
        receiveTimeout: rpcTimeout.duration,
        headers: {
          Headers.contentTypeHeader: setuContentType,
          SetuHeaders.id: id.toString(),
          SetuHeaders.timeout: rpcTimeout.toString(),
        },
      ),
    );

    final contentType = response.headers.value(Headers.contentTypeHeader);
    if (contentType != setuContentType) {
      throw Exception('unexpected content-type: ${contentType ?? "none"}');
    }
    return response.data!;
  }
}

