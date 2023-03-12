/// Concatenate the first 2 bytes and the last 2 bytes of a slice into a single
/// 32-bit integer as an efficient hashing function of JS keywords.
/// This approach is backed by the observation that the byte-sequence formed by taking
/// the first 2 and last 2 bytes of JS keywords are unique.
///
/// SAFETY:
/// key.len() >= 2
#[inline]
pub unsafe fn extract_first_and_last_two_bytes(key: &[u8]) -> u32 {
    unsafe fn read_u16(input: &[u8]) -> u16 {
        let (input, _) = input.split_at(std::mem::size_of::<u16>());
        u16::from_be_bytes(TryInto::<[u8; 2]>::try_into(input).unwrap())
    }
    // read first 2 bytes in a u16
    let first = read_u16(key);
    let last_bytes = &key[key.len() - 2..];
    let last = read_u16(last_bytes);
    u32::from(first) | (u32::from(last)) << 16
}

#[inline]
pub fn hash_u32(input: u32, seed: u32) -> u32 {
    const MAGIC: u64 = 887_987_685;
    let hash = input ^ seed;
    ((u64::from(hash) * MAGIC) >> 32) as u32
}
