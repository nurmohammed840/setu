import 'package:setu_client/http/client.dart';

void main() async {
  final fetch = createHttpClient();

  final res = await fetch.get("https://127.0.0.1:4433");

  print('Status: ${res.statusCode}');
  print('Headers: ${res.headers}');
  print('Body: ${res.data}');

  fetch.close(force: true);
}
