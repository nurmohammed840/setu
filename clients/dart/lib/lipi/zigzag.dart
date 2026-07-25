BigInt zigzagEncode(BigInt n) {
  return (n << 1) ^ (n >> 63);
}

int zigzagDecode(BigInt n) {
  return ((n >> 1) ^ -(n & BigInt.one)).toInt();
}