# IEEE80211-rs
IEEE80211-rs is a library for dealing with IEEE 802.11 frames. It's highly experimental and unstable. It shouldn't be used in production yet, since the architecture may change.
As soon as the first version is released, some level of API stability is to be expected.

## Note
This library is currently only maintained by one person. I try to add stuff here as frequently as I can, but it's usually a byproduct of my main projects like [GraCe](https://github.com/Frostie314159/grace), which started this project. My current focus is working on the [ESP32-Open-MAC](https://esp32-open-mac.be/) project, which seeks to reverse engineer the PHY and MAC layers of the ESP32 and replace the proprietary WiFi stack. In the future I would like to write a WiFi stack in Rust, for which this library will be used. If you need certain features, which are still missing, feel free to open an issue. If they aren't too complex (unlike the RSNE), I should get them merged in under a week. If you want to implement them yourself, I'm happy to provide assistance where it's needed.

## Supported features
### Frame format
Only Protocol version zero is supported. This means no S1G (802.11ah or HaLow) for now.
#### Management Frames
- Association Request / Response
- Probe Request / Response
- Beacon
- ATIM
- Disassociation
- Deauthentication
- Action(No ACK)
#### Control Frames
- RTS
- CTS
- Ack
#### Data Frames
The data frame implementation can handle any frames.
#### Elements
- SSID
- Supported Rates
- DSSS Parameter Set
- IBSS Parameter Set
- BSS Load
- HT Capabilities/Operation
- Extended Supported Rates
- RSN
- VHT Capabilities/Operation
- Vendor Specific
- OWE Transition
- Mesh ID


## Contents
This section lists, the range of things handled by this library.
### Serialization and Deserialization
Serialization and deserialization capabilities were the first thing, that I started implementing and is likely to be the most mature part of the library.

#### Architecture
Each frame type is represented by it's own type, like the `BeaconFrame`, and can be read and written through [scroll](https://github.com/m4b/scroll). If used in this way, it's the users responsibility to ensure, that the  correct frame type is being read. If you have to handle multiple frame types, you can use the `match_frames!` macro, to handle different frame types.

## Optimization
This library can benefit greatly from Profile-Guide-Optimization(PGO), which can be used through [cargo-pgo](https://github.com/Kobzol/cargo-pgo). For further details, see the [issue](https://github.com/Frostie314159/ieee80211-rs/issues/3) created by [zamazan4jk](https://github.com/zamazan4ik), so thanks a lot to him.

## Other libraries
- [AirCap](https://github.com/Frostie314159/aircap) | WiFi capturing on linux
- [Rtap](https://github.com/Frostie314159/rtap) | Radiotap parsing

## no_std and zerocopy
This library doesn't require allocations and also doesn't copy the data around. It is designed to be usable even on embedded devices.

## unsafe code
The usage of unsafe code inside this library is forbidden.

## Contributing
Nice, that you want to contribute! The first thing you should do is, to check if someones already working on it. If that isn't the case, fork the repo and create a draft PR. That way, others can see that someone is working on the feature. Once you think that your code is ready to be merged, remove the draft status and we'll review it.
### Guidelines
Every line of code should be tested. At the time of writing this(April 2024), this isn't yet the case for every struct, but we're working towards it. For testing reading and writing, there are macros, which auto generate these tests. For everything else, you should test every function for their expected outputs.

You should try and use your own code in practice, since that's the only way to truly test it. You could for example build a sniffer, which makes your code process real world data.

## Access to IEEE 802.11
IEEE 802.11 is copyrighted by the IEEE, which is why we can't just share it publicly. You can however acquire your own copy of the standard for free through the IEEE's [GET program](https://ieeexplore.ieee.org/browse/standards/get-program/page). It requires a free account and only grants you access to currently active standards, which are older than six months. This means, that we'll probably be able to implement IEEE 802.11be related functionality by mid summer 2025, since it's expected to be ratified in Q4 2024.

## MSRV
The current MSRV is version 1.65.0.

## License
This library is licensed under the MIT or Apache-2.0 License at your option. 
