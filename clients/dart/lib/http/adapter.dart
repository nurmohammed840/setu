import 'package:dio/dio.dart';

void configureAdapter(Dio dio) {
  dio.httpClientAdapter = HttpClientAdapter();
}