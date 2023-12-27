use core::{
    iter::{Copied, Map},
    slice::Iter,
};

/// The default rate iterator returned, when parsing the [SupportedRatesTLV].
pub type RateReadIterator<'a, Rate> = Map<Copied<Iter<'a, u8>>, fn(u8) -> Rate>;
