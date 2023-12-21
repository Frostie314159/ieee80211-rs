# IEEE80211-rs
IEEE80211-rs is a library for serializing and deserializing IEEE 802.11 frames. It's highly experimental and unstable. It shouldn't be used in production yet, since the architecture may change.
As soon as the first version is released, some level of API stability is to be expected.

## Architecture
Deserializing a `Frame` will destructure the frame into it's most elementary form. This means, that the fixed fields of - for example - an action frame will all be deserialized. The tagged fields however are not deserialized immediately and remain as a byte slice, which can be converted into an `Iterator<Item = IEEE80211TLV<'_>>`. At first parsing everything immediately, might seem terribly slow, however this library is zerocopy and the actual processing time is minimal.
*Benchmarks will follow soon.*

## no_std and zerocopy
This library doesn't require allocations and also doesn't copy the data around. It is designed to be usable even on embedded devices.
## unsafe code
The usage of unsafe code inside this library is forbidden.
## License
This library is licensed under the MIT or Apache-2.0 License at your option. 
