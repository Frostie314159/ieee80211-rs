fn xswap(l: u32) -> u32 {
    ((l & 0xff00ff00) >> 8) | ((l & 0x00ff00ff) << 8)
}
/// Compute the michael block function.
pub fn michael_block_function(l: u32, r: u32) -> (u32, u32) {
    let mut r = r ^ l.rotate_left(17);
    let mut l = l.wrapping_add(r);
    r ^= xswap(l);
    l = l.wrapping_add(r);
    r ^= l.rotate_left(3);
    l = l.wrapping_add(r);
    r ^= l.rotate_right(2);
    l = l.wrapping_add(r);
    (l, r)
}
/// Compute the michael MIC of the bytes, with the key.
pub fn michael(key: u64, bytes: &[u8]) -> u64 {
    // NOTE: This implementation is partially adapted from https://github.com/torvalds/linux/blob/master/net/mac80211/michael.c
    let (mut l, mut r) = (((key >> 32) as u32).to_be(), (key as u32).to_be());

    let blocks = bytes.len() / 4;
    let left = bytes.len() % 4;

    for i in 0..blocks {
        let block = &bytes[(i * 4)..][..4];
        let block = u32::from_le_bytes(block.try_into().unwrap());
        l ^= block;
        (l, r) = michael_block_function(l, r);
    }
    let mut block = [0x00; 4];
    block[..left].copy_from_slice(&bytes[(blocks * 4)..][..left]);
    block[left] = 0x5a;
    let block = u32::from_le_bytes(block);
    l ^= block;
    (l, r) = michael_block_function(l, r);

    // Run the final block, which is all zeroes.
    (l, r) = michael_block_function(l, r);

    let mut mic = [0x00; 8];
    mic[..4].copy_from_slice(l.to_le_bytes().as_slice());
    mic[4..8].copy_from_slice(r.to_le_bytes().as_slice());
    u64::from_be_bytes(mic)
}
