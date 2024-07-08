mod beacon;
macro_rules! gen_frame_rw_test {
    ($test_name:ident, $frame_type:ty, $expected_frame:expr, $expected_bytes:expr) => {
        #[test]
        fn $test_name() {
            use ::scroll::{Pread, Pwrite, ctx::MeasureWith};

            let read_frame = $expected_bytes.pread::<$frame_type>(0).unwrap();
            assert_eq!(read_frame, $expected_frame, "The {} read from the supplied bytes didn't match, what was expected. 
                Either check the test case for correctness or the TryFromCtx implementation.", stringify!($frame_type));

            let expected_length = read_frame.measure_with(&());
            let mut buf = ::std::vec![0x00; expected_length];
            let written = buf.pwrite($expected_frame, 0).unwrap();
            assert_eq!(written, expected_length, "The amount of bytes, written by TryIntoCtx, for {}, didn't match the amount returned by MeasureWith. 
                Either check the test case for correctness or the TryIntoCtx and or MeasureWith implementation.", stringify!($frame_type));
            assert_eq!(buf, $expected_bytes, "The bytes, written by TryIntoCtx, for {}, didn't match what was expected.
                Either check the test case for correctness or the TryIntoCtx implementation.", stringify!($frame_type));
        }
    };
}
