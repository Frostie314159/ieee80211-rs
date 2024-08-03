use ieee80211::crypto::{michael, michael_block_function};

#[test]
fn test_michael() {
    assert_eq!(michael(0, b""), 0x82925c1ca1d130b8);
    assert_eq!(michael(0x82925c1ca1d130b8, b"M"), 0x434721ca40639b3f);
    assert_eq!(michael(0x434721ca40639b3f, b"Mi"), 0xe8f9becae97e5d29);
    assert_eq!(michael(0xe8f9becae97e5d29, b"Mic"), 0x90038fc6cf13c1db);
    assert_eq!(michael(0x90038fc6cf13c1db, b"Mich"), 0xd55e100510128986);
    assert_eq!(michael(0xd55e100510128986, b"Michael"), 0x0a942b124ecaa546);
}

#[test]
fn test_block_function() {
    assert_eq!(michael_block_function(0, 0), (0, 0));
    assert_eq!(michael_block_function(0, 1), (0xc00015a8, 0xc0000b95));
    assert_eq!(michael_block_function(1, 0), (0x6b519593, 0x572b8b8a));
    assert_eq!(
        michael_block_function(0x01234567, 0x83659326),
        (0x441492c2, 0x1d8427ed)
    );
    assert_eq!(
        (0..1000).fold((1, 0), |(l, r), _| michael_block_function(l, r)),
        (0x9f04c4ad, 0x2ec6c2bf)
    );
}
