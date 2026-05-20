export function zigzagEncode(num: bigint): bigint {
  return (num << 1n) ^ (num >> 63n);
}

export function zigzagDecode(num: bigint): bigint {
  return (num >> 1n) ^ (-(num & 1n));
}
