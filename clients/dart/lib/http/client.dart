import 'package:dio/dio.dart';
import 'adapter.dart'
    if (dart.library.io) 'native.dart'
    if (dart.library.js_interop) 'web.dart';

Dio createHttpClient() {
  final dio = Dio();

  configureAdapter(dio);

  return dio;
}
