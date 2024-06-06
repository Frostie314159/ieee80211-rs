mod ssid;
mod dsss_parameter_set;

#[macro_export]
macro_rules! gen_element_rw_test {
    ($test_name:ident, $element:ty, $expected_element:expr, $expected_bytes:expr) => {
        #[test]
        fn $test_name() {
            use ::scroll::{Pread, Pwrite, ctx::MeasureWith};

            let read_element = $expected_bytes.pread::<$element>(0).unwrap();
            assert_eq!(read_element, $expected_element, "The {} read from the supplied bytes didn't match, what was expected. 
                Either check the test case for correctness or the TryFromCtx implementation.", stringify!($element));

            let expected_length = read_element.measure_with(&());
            let mut buf = ::std::vec![0x00; expected_length];
            let written = buf.pwrite($expected_element, 0).unwrap();
            assert_eq!(written, expected_length, "The amount of bytes, written by TryIntoCtx, for {}, didn't match the amount returned by MeasureWith. 
                Either check the test case for correctness or the TryIntoCtx and or MeasureWith implementation.", stringify!($element));
            assert_eq!(buf, $expected_bytes, "The bytes, written by TryIntoCtx, for {}, didn't match what was expected.
                Either check the test case for correctness or the TryIntoCtx implementation.", stringify!($element));
        }
    };
}
