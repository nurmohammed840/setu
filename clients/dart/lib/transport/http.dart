import 'dart:typed_data';

import 'package:dio/dio.dart';

import 'package:setu_client/errors.dart';
import 'package:setu_client/http/client.dart';
import 'package:setu_client/setu/http_headers.dart';
import 'package:setu_client/timeout.dart';
import 'package:setu_client/utils/common.dart';

class RPC {
  static var uri = Uri.parse('https://localhost:443/');
  static var timeout = Timeout.minute(2);
  static var dio = createHttpClient();

  static Future<Stream<Uint8List>> call({
    required int id,
    required Object body,

    Dio? dio,
    Uri? uri,
    Timeout? timeout,
    CancelToken? cancelToken,
  }) async {
    final headers = {
      Headers.contentTypeHeader: setuContentType,
      SetuHeaders.id: id.toString(),
    };

    if (timeout != null) {
      headers[SetuHeaders.timeout] = timeout.toString();
    }

    final response = await (dio ?? RPC.dio).post(
      (uri ?? RPC.uri).toString(),
      data: body,
      cancelToken: cancelToken,
      options: Options(
        responseType: ResponseType.stream,
        sendTimeout: timeout?.duration,
        receiveTimeout: timeout?.duration,
        headers: {
          Headers.contentTypeHeader: setuContentType,
          SetuHeaders.id: id.toString(),
        },
      ),
    );

    final contentType = response.headers.value(Headers.contentTypeHeader);
    assertExpr(
      contentType == setuContentType,
      error: ProtocolError.new,
      message: () => 'unexpected content-type: ${contentType ?? "none"}',
    );

    return response.data.stream;
  }
}
