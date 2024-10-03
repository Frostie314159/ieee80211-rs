#![deny(unused)]

mod aid;
#[cfg(feature = "crypto")]
mod crypto;
mod elements;
mod frames;
mod issues;
#[macro_export]
macro_rules! roundtrip_test {
    ($test_name:ident, $read_type:ty, $expected_read:expr, $expected_bytes:expr) => {
        #[test]
        fn $test_name() {
            use ::scroll::{Pread, Pwrite, ctx::MeasureWith};

            let read = $expected_bytes.pread::<$read_type>(0).unwrap();
            assert_eq!(read, $expected_read, "The {} read from the supplied bytes didn't match, what was expected. 
                Either check the test case for correctness or the TryFromCtx implementation.", stringify!($read_type));

            let expected_length = read.measure_with(&());
            let mut buf = ::std::vec![0x00; expected_length];
            let written = buf.pwrite($expected_read, 0).unwrap();
            assert_eq!(written, expected_length, "The amount of bytes, written by TryIntoCtx, for {}, didn't match the amount returned by MeasureWith. 
                Either check the test case for correctness or the TryIntoCtx and or MeasureWith implementation.", stringify!($read_type));
            assert_eq!(buf, $expected_bytes, "The bytes, written by TryIntoCtx, for {}, didn't match what was expected.
                Either check the test case for correctness or the TryIntoCtx implementation.", stringify!($read_type));
        }
    };
}
