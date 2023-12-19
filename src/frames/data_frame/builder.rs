use core::marker::PhantomData;

use mac_parser::MACAddress;

use crate::{frag_seq_info::FragSeqInfo, frame_control_field::FCFFlags, type_state::*};

use self::type_state::{
    AddressExtractor, AddressTwoIsSA, Data, DataFrameCategory, DataNull, HasPayload, Payload, QoS,
    QoSNull,
};

use super::{amsdu::AMSDUPayload, DataFrame, DataFrameSubtype};

#[allow(dead_code)]
pub mod type_state {

    use mac_parser::MACAddress;
    use scroll::ctx::{MeasureWith, TryIntoCtx};

    use crate::{
        frames::data_frame::amsdu::AMSDUPayload,
        type_state::{NeitherToNorFromDS, ToDS},
    };

    pub trait DataFrameCategory {
        const UPPER_TWO_BITS: u8;
    }
    pub trait HasPayload {}
    pub trait NoPayload {}
    macro_rules! data_frame_category {
        ($category_name:ident, $trait_name:ident, $upper_two_bits:expr) => {
            pub struct $category_name;
            impl DataFrameCategory for $category_name {
                const UPPER_TWO_BITS: u8 = $upper_two_bits;
            }
            impl $trait_name for $category_name {}
        };
    }
    data_frame_category!(Data, HasPayload, 0b00);
    data_frame_category!(DataNull, NoPayload, 0b01);
    data_frame_category!(QoS, HasPayload, 0b10);
    data_frame_category!(QoSNull, NoPayload, 0b11);
    pub trait Payload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + Copy {
        const IS_AMSDU: bool;
    }
    impl Payload for &'_ [u8] {
        const IS_AMSDU: bool = false;
    }
    impl Payload for AMSDUPayload<'_> {
        const IS_AMSDU: bool = true;
    }
    pub struct PayloadLengthExtractor<PayloadType: Payload>(pub PayloadType);
    impl PayloadLengthExtractor<&'_ [u8]> {
        pub const fn length_in_bytes(&self) -> usize {
            self.0.len()
        }
    }
    impl PayloadLengthExtractor<AMSDUPayload<'_>> {
        pub const fn length_in_bytes(&self) -> usize {
            self.0.length_in_bytes()
        }
    }

    pub trait AddressTwoIsSA {}
    impl AddressTwoIsSA for NeitherToNorFromDS {}
    impl AddressTwoIsSA for ToDS {}

    pub(crate) struct AddressExtractor<Address>(pub Address);
    impl AddressExtractor<()> {
        pub(crate) const fn get_address(&self) -> Option<MACAddress> {
            None
        }
    }
    impl AddressExtractor<MACAddress> {
        pub(crate) const fn get_address(&self) -> Option<MACAddress> {
            Some(self.0)
        }
    }
}
#[doc(hidden)]
/// A type state based data frame builder.
pub struct DataFrameBuilderInner<
    'a,
    DS,
    Category,
    PayloadType,
    Address1,
    Address2,
    Address3,
    Address4,
> {
    address_1: Address1,
    address_2: Address2,
    address_3: Address3,
    address_4: Address4,
    payload: Option<PayloadType>,
    _phantom: PhantomData<(DS, Category, &'a ())>,
}
impl<
        DS,
        Category,
        PayloadType: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
        Address4: Copy,
    > DataFrameBuilderInner<'_, DS, Category, PayloadType, Address1, Address2, Address3, Address4>
{
    #[inline]
    const fn change_type_state<'a, NewDS, NewCategory>(
        self,
    ) -> DataFrameBuilderInner<
        'a,
        NewDS,
        NewCategory,
        PayloadType,
        Address1,
        Address2,
        Address3,
        Address4,
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl DataFrameBuilderInner<'_, (), (), (), (), (), (), ()> {
    #[inline]
    pub const fn new<'a>() -> DataFrameBuilderInner<'a, (), (), (), (), (), (), ()> {
        DataFrameBuilderInner {
            address_1: (),
            address_2: (),
            address_3: (),
            address_4: (),
            payload: None,
            _phantom: PhantomData,
        }
    }
    pub const fn neither_to_nor_from_ds<'a>(
        self,
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn to_ds<'a>(self) -> DataFrameBuilderInner<'a, ToDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn from_ds<'a>(self) -> DataFrameBuilderInner<'a, FromDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn to_and_from_ds<'a>(
        self,
    ) -> DataFrameBuilderInner<'a, ToAndFromDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
}
impl<DS> DataFrameBuilderInner<'_, DS, (), (), (), (), (), ()> {
    pub const fn category_data<'a>(
        self,
    ) -> DataFrameBuilderInner<'a, DS, Data, (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn category_data_null<'a>(
        self,
    ) -> DataFrameBuilderInner<'a, DS, DataNull, (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn category_qos<'a>(self) -> DataFrameBuilderInner<'a, DS, QoS, (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn category_qos_null<'a>(
        self,
    ) -> DataFrameBuilderInner<'a, DS, QoSNull, (), (), (), (), ()> {
        self.change_type_state()
    }
}
impl<DS, Category: HasPayload + DataFrameCategory>
    DataFrameBuilderInner<'_, DS, Category, (), (), (), (), ()>
{
    pub const fn payload(
        self,
        payload: &[u8],
    ) -> DataFrameBuilderInner<'_, DS, Category, &[u8], (), (), (), ()> {
        DataFrameBuilderInner {
            address_1: (),
            address_2: (),
            address_3: (),
            address_4: (),
            payload: Some(payload),
            _phantom: PhantomData,
        }
    }
}
impl<DS> DataFrameBuilderInner<'_, DS, QoS, (), (), (), (), ()> {
    pub const fn payload_amsdu(
        self,
        payload: AMSDUPayload<'_>,
    ) -> DataFrameBuilderInner<'_, DS, QoS, AMSDUPayload<'_>, (), (), (), ()> {
        DataFrameBuilderInner {
            address_1: (),
            address_2: (),
            address_3: (),
            address_4: (),
            payload: Some(payload),
            _phantom: PhantomData,
        }
    }
}
impl<DS, Category, PayloadType: Copy, Address2: Copy, Address3: Copy, Address4: Copy>
    DataFrameBuilderInner<'_, DS, Category, PayloadType, (), Address2, Address3, Address4>
{
    pub const fn receiver_address<'a>(
        self,
        receiver_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        DS,
        Category,
        PayloadType,
        MACAddress,
        Address2,
        Address3,
        Address4,
    > {
        DataFrameBuilderInner {
            address_1: receiver_address,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<DS, Category, PayloadType: Copy, Address1: Copy, Address3: Copy, Address4: Copy>
    DataFrameBuilderInner<'_, DS, Category, PayloadType, Address1, (), Address3, Address4>
{
    pub const fn transmitter_address<'a>(
        self,
        transmitter_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        DS,
        Category,
        PayloadType,
        Address1,
        MACAddress,
        Address3,
        Address4,
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: transmitter_address,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, PayloadType: Copy, Address2: Copy, Address3: Copy>
    DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, (), Address2, Address3, ()>
{
    pub const fn destination_address(
        self,
        destination_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        NeitherToNorFromDS,
        Category,
        PayloadType,
        MACAddress,
        Address2,
        Address3,
        (),
    > {
        DataFrameBuilderInner {
            address_1: destination_address,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, PayloadType: Copy, Address2: Copy, Address3: Copy>
    DataFrameBuilderInner<'a, FromDS, Category, PayloadType, (), Address2, Address3, ()>
{
    pub const fn destination_address(
        self,
        destination_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        NeitherToNorFromDS,
        Category,
        PayloadType,
        MACAddress,
        Address2,
        Address3,
        (),
    > {
        DataFrameBuilderInner {
            address_1: destination_address,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy, Address2: Copy, Address4: Copy>
    DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, (), Address4>
{
    pub const fn destination_address(
        self,
        destination_address: MACAddress,
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, MACAddress, Address4>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: destination_address,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy, Address2: Copy, Address4: Copy>
    DataFrameBuilderInner<'a, ToAndFromDS, Category, &'a [u8], Address1, Address2, (), Address4>
{
    pub const fn destination_address(
        self,
        destination_address: MACAddress,
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, MACAddress, Address4>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: destination_address,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<DS: AddressTwoIsSA, Category, PayloadType: Copy, Address1: Copy, Address3: Copy>
    DataFrameBuilderInner<'_, DS, Category, PayloadType, Address1, (), Address3, ()>
{
    pub const fn source_address<'a>(
        self,
        source_address: MACAddress,
    ) -> DataFrameBuilderInner<'a, DS, Category, PayloadType, Address1, MACAddress, Address3, ()>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: source_address,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy, Address2: Copy>
    DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, Address2, (), ()>
{
    pub const fn source_address(
        self,
        source_address: MACAddress,
    ) -> DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, Address2, MACAddress, ()>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: source_address,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, PayloadType: Copy, Address1: Copy, Address2: Copy>
    DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, Address1, Address2, (), ()>
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        NeitherToNorFromDS,
        Category,
        PayloadType,
        Address1,
        Address2,
        MACAddress,
        (),
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: bssid,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy, Address3: Copy>
    DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, (), Address3, ()>
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, MACAddress, Address3, ()>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: bssid,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address2: Copy, Address3: Copy>
    DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], (), Address2, Address3, ()>
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], MACAddress, Address2, Address3, ()>
    {
        DataFrameBuilderInner {
            address_1: bssid,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address2: Copy>
    DataFrameBuilderInner<'a, ToDS, Category, AMSDUPayload<'a>, (), Address2, (), ()>
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToDS,
        Category,
        AMSDUPayload<'a>,
        MACAddress,
        Address2,
        MACAddress,
        (),
    > {
        DataFrameBuilderInner {
            address_1: bssid,
            address_2: self.address_2,
            address_3: bssid,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy, Address2: Copy, Address3: Copy>
    DataFrameBuilderInner<
        'a,
        ToAndFromDS,
        Category,
        &'a [u8],
        Address1,
        Address2,
        Address3,
        MACAddress,
    >
{
    pub const fn source_address(
        self,
        source_address: MACAddress,
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, Address3, MACAddress>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: source_address,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy, Address2: Copy>
    DataFrameBuilderInner<'a, ToAndFromDS, Category, AMSDUPayload<'a>, Address1, Address2, (), ()>
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToDS,
        Category,
        AMSDUPayload<'a>,
        Address1,
        Address2,
        MACAddress,
        MACAddress,
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: bssid,
            address_4: bssid,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category, Address1: Copy>
    DataFrameBuilderInner<'a, FromDS, Category, AMSDUPayload<'a>, Address1, (), (), ()>
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToDS,
        Category,
        AMSDUPayload<'a>,
        Address1,
        MACAddress,
        MACAddress,
        (),
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: bssid,
            address_3: bssid,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<DS: DSField, Category: DataFrameCategory, PayloadType: Payload>
    DataFrameBuilderInner<'_, DS, Category, PayloadType, MACAddress, MACAddress, MACAddress, ()>
{
    #[inline]
    pub const fn build<'a>(self) -> DataFrame<PayloadType> {
        DataFrame {
            sub_type: DataFrameSubtype::from_representation(Category::UPPER_TWO_BITS << 2),
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: AddressExtractor(self.address_4).get_address(),
            fcf_flags: FCFFlags {
                from_ds: DS::FROM_DS,
                to_ds: DS::TO_DS,
                more_fragments: false,
                retry: false,
                pwr_mgt: false,
                more_data: false,
                protected: false,
                htc_plus_order: PayloadType::IS_AMSDU,
            },
            duration: 0,
            frag_seq_info: FragSeqInfo {
                fragment_number: 0,
                sequence_number: 0,
            },
            qos: None,
            ht_control: None,
            payload: self.payload,
        }
    }
}
pub type DataFrameBuilder<'a> = DataFrameBuilderInner<'a, (), (), (), (), (), (), ()>;

#[test]
fn test() {
    let _data_frame = DataFrameBuilder::new()
        .neither_to_nor_from_ds()
        .category_qos();
}
