import 'package:dio/browser.dart';
import 'package:dio/dio.dart';

void configureAdapter(Dio dio) {
  dio.httpClientAdapter = BrowserHttpClientAdapter();
}
