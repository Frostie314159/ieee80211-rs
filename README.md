# IEEE80211-rs
IEEE80211-rs is a library for dealing with IEEE 802.11 frames. It's highly experimental and unstable. It shouldn't be used in production yet, since the architecture may change.
As soon as the first version is released, some level of API stability is to be expected.

## Contents
This section lists, the range of things handled by this library.
### Serialization and Deserialization
Serialization and deserialization capabilities were the first thing, that I started implementing and is likely to be the most mature part of the library.

#### Architecture
Deserializing an `IEEE80211Frame` will destructure the frame into it's most elementary form. This means, that the fixed fields of - for example - an action frame will all be deserialized. The tagged fields however are not deserialized immediately and remain as an `Elements` struct, which can be used to extract the different element types. At first parsing everything immediately, might seem terribly slow, however this library is zerocopy and the actual processing time is minimal.

#### Benchmarks
Hardware: Framework 13 with Intel i5 1240p
|benchmark|time in ns|notes|
-- | -- | --
beacon_read | 40.2 | --
beacon_write | 43.4 | This is using the element chain approach.
action_vendor_read | 31.8 | --
action_vendor_write | 26.3 | --
qos_data_read | 20.5 | --
qos_data_write | 25 | --


## no_std and zerocopy
This library doesn't require allocations and also doesn't copy the data around. It is designed to be usable even on embedded devices.

## unsafe code
The usage of unsafe code inside this library is forbidden.

## Contributing
Nice, that you want to contribute! The first thing you should do is, to check if someones already working on it. If that isn't the case, fork the repo and create a draft PR. That way, others can see that someone is working on the feature. Once you think that your code is ready to be merged, remove the draft status and we'll review it.
### Guidelines
Every line of code should be tested. At the time of writing this(April 2024), this isn't yet the case for every struct, but we're working towards it. For testing reading and writing, there are macros, which auto generate these tests. For everything else, you should test every function for their expected outputs.

You should try and use your own code in practice, since that's the only way to truly test it. You could for example build a sniffer, which makes your code process real world data.

## License
This library is licensed under the MIT or Apache-2.0 License at your option. 
