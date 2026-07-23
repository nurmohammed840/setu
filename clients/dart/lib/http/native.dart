// Android, iOS, Windows, macOS, Linux
import 'package:dio/dio.dart';
import 'package:dio_http2_adapter/dio_http2_adapter.dart';

void configureAdapter(Dio dio) {
  dio.httpClientAdapter = Http2Adapter(
    ConnectionManager(
      idleTimeout: const Duration(seconds: 60),
      // Ignoring bad certificate
      onClientCreate: (_, config) => config.onBadCertificate = (_) => true,
    ),
  );
}
