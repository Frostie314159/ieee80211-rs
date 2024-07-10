use core::marker::PhantomData;
use mac_parser::MACAddress;
use scroll::ctx::TryIntoCtx;

use crate::common::*;

use self::type_state::{Data, DataFrameCategory, DataNull, HasPayload, QoS, QoSNull};

use super::{amsdu::AMSDUPayload, header::DataFrameHeader, DataFrame};

pub mod type_state {

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
}
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
    address_4: Option<MACAddress>,
    payload: Option<PayloadType>,
    fcf_flags: FCFFlags,
    _phantom: PhantomData<(&'a (), DS, Category, Address4)>,
}
impl<
        'a,
        DS,
        Category,
        PayloadType: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
        Address4: Copy,
    > DataFrameBuilderInner<'a, DS, Category, PayloadType, Address1, Address2, Address3, Address4>
{
    #[inline]
    const fn change_type_state<NewDS, NewCategory>(
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a> DataFrameBuilderInner<'a, (), (), (), (), (), (), ()> {
    #[inline]
    pub const fn new() -> DataFrameBuilderInner<'a, (), (), (), (), (), (), ()> {
        DataFrameBuilderInner {
            address_1: (),
            address_2: (),
            address_3: (),
            address_4: None,
            payload: None,
            fcf_flags: FCFFlags::new(),
            _phantom: PhantomData,
        }
    }
    pub const fn neither_to_nor_from_ds(
        self,
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn to_ds(self) -> DataFrameBuilderInner<'a, ToDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn from_ds(self) -> DataFrameBuilderInner<'a, FromDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn to_and_from_ds(
        self,
    ) -> DataFrameBuilderInner<'a, ToAndFromDS, (), (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn more_fragments(mut self) -> Self {
        self.fcf_flags = self.fcf_flags.with_more_fragments(true);
        self
    }
    pub const fn retry(mut self) -> Self {
        self.fcf_flags = self.fcf_flags.with_retry(true);
        self
    }
    pub const fn power_management(mut self) -> Self {
        self.fcf_flags = self.fcf_flags.with_pwr_mgmt(true);
        self
    }
    pub const fn more_data(mut self) -> Self {
        self.fcf_flags = self.fcf_flags.with_more_data(true);
        self
    }
    pub const fn protected(mut self) -> Self {
        self.fcf_flags = self.fcf_flags.with_protected(true);
        self
    }
    pub const fn order(mut self) -> Self {
        self.fcf_flags = self.fcf_flags.with_order(true);
        self
    }
}
impl<'a, DS> DataFrameBuilderInner<'a, DS, (), (), (), (), (), ()> {
    pub const fn category_data(self) -> DataFrameBuilderInner<'a, DS, Data, (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn category_data_null(
        self,
    ) -> DataFrameBuilderInner<'a, DS, DataNull, (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn category_qos(self) -> DataFrameBuilderInner<'a, DS, QoS, (), (), (), (), ()> {
        self.change_type_state()
    }
    pub const fn category_qos_null(
        self,
    ) -> DataFrameBuilderInner<'a, DS, QoSNull, (), (), (), (), ()> {
        self.change_type_state()
    }
}
impl<'a, DS, Category: HasPayload + DataFrameCategory>
    DataFrameBuilderInner<'a, DS, Category, (), (), (), (), ()>
{
    pub const fn payload<Payload: TryIntoCtx + 'a>(
        self,
        payload: Payload,
    ) -> DataFrameBuilderInner<'a, DS, Category, Payload, (), (), (), ()> {
        DataFrameBuilderInner {
            address_1: (),
            address_2: (),
            address_3: (),
            address_4: None,
            payload: Some(payload),
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a, DS> DataFrameBuilderInner<'a, DS, QoS, (), (), (), (), ()> {
    pub const fn payload_amsdu<SubFrames>(
        self,
        sub_frames: SubFrames,
    ) -> DataFrameBuilderInner<'a, DS, QoS, AMSDUPayload<SubFrames>, (), (), (), ()> {
        DataFrameBuilderInner {
            address_1: (),
            address_2: (),
            address_3: (),
            address_4: None,
            payload: Some(AMSDUPayload { sub_frames }),
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a, DS, Category, PayloadType: Copy, Address2: Copy, Address3: Copy, Address4: Copy>
    DataFrameBuilderInner<'a, DS, Category, PayloadType, (), Address2, Address3, Address4>
{
    pub const fn receiver_address(
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a, DS, Category, PayloadType: Copy, Address1: Copy, Address3: Copy, Address4: Copy>
    DataFrameBuilderInner<'a, DS, Category, PayloadType, Address1, (), Address3, Address4>
{
    pub const fn transmitter_address(
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        PayloadType: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
    >
    DataFrameBuilderInner<
        'a,
        NeitherToNorFromDS,
        Category,
        PayloadType,
        Address1,
        Address2,
        Address3,
        (),
    >
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
    pub const fn source_address(
        self,
        source_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        NeitherToNorFromDS,
        Category,
        PayloadType,
        Address1,
        MACAddress,
        Address3,
        (),
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: source_address,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
    pub const fn bssid(
        self,
        bssid: MACAddress,
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
            address_1: bssid,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        PayloadType: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
    > DataFrameBuilderInner<'a, FromDS, Category, PayloadType, Address1, Address2, Address3, ()>
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy>
    DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, Address2, Address3, ()>
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, MACAddress, Address3, ()>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: bssid,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        Payload: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
    >
    DataFrameBuilderInner<
        'a,
        FromDS,
        Category,
        AMSDUPayload<Payload>,
        Address1,
        Address2,
        Address3,
        (),
    >
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        FromDS,
        Category,
        AMSDUPayload<Payload>,
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        PayloadType: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
    > DataFrameBuilderInner<'a, ToDS, Category, PayloadType, Address1, Address2, Address3, ()>
{
    pub const fn source_address(
        self,
        source_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        NeitherToNorFromDS,
        Category,
        PayloadType,
        Address1,
        MACAddress,
        Address3,
        (),
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: source_address,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy>
    DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, Address3, ()>
{
    pub const fn destination_address(
        self,
        destination_address: MACAddress,
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, MACAddress, ()>
    {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: destination_address,
            address_4: self.address_4,
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        Payload: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
    >
    DataFrameBuilderInner<
        'a,
        ToDS,
        Category,
        AMSDUPayload<Payload>,
        Address1,
        Address2,
        Address3,
        (),
    >
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToDS,
        Category,
        AMSDUPayload<Payload>,
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
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
        Address4: Copy,
    >
    DataFrameBuilderInner<
        'a,
        ToAndFromDS,
        Category,
        &'a [u8],
        Address1,
        Address2,
        Address3,
        Address4,
    >
{
    pub const fn destination_address(
        self,
        destination_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToAndFromDS,
        Category,
        &'a [u8],
        Address1,
        Address2,
        MACAddress,
        Address4,
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: destination_address,
            address_4: self.address_4,
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
    pub const fn source_address(
        self,
        source_address: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToAndFromDS,
        Category,
        &'a [u8],
        Address1,
        Address2,
        Address3,
        MACAddress,
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: Some(source_address),
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<
        'a,
        Category: DataFrameCategory,
        Payload: Copy,
        Address1: Copy,
        Address2: Copy,
        Address3: Copy,
        Address4: Copy,
    >
    DataFrameBuilderInner<
        'a,
        ToAndFromDS,
        Category,
        AMSDUPayload<Payload>,
        Address1,
        Address2,
        Address3,
        Address4,
    >
{
    pub const fn bssid(
        self,
        bssid: MACAddress,
    ) -> DataFrameBuilderInner<
        'a,
        ToAndFromDS,
        Category,
        AMSDUPayload<Payload>,
        Address1,
        Address2,
        MACAddress,
        MACAddress,
    > {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: bssid,
            address_4: Some(bssid),
            payload: self.payload,
            fcf_flags: self.fcf_flags,
            _phantom: PhantomData,
        }
    }
}
impl<'a, DS: DSField, Category: DataFrameCategory, PayloadType: Copy>
    DataFrameBuilderInner<
        'a,
        DS,
        Category,
        PayloadType,
        MACAddress,
        MACAddress,
        MACAddress,
        MACAddress,
    >
{
    #[inline]
    pub const fn build(self) -> DataFrame<'a, PayloadType> {
        let header = DataFrameHeader {
            subtype: DataFrameSubtype::from_bits(Category::UPPER_TWO_BITS << 2),
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            fcf_flags: self.fcf_flags,
            duration: 0,
            frag_seq_info: SequenceControl::new(),
            qos: None,
            ht_control: None,
        };
        DataFrame::<'a, PayloadType> {
            header,
            payload: self.payload,
            _phantom: PhantomData,
        }
    }
}
impl<'a> Default for DataFrameBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
pub type DataFrameBuilder<'a> = DataFrameBuilderInner<'a, (), (), (), (), (), (), ()>;

#[test]
fn test() {
    use crate::frames::data_frame::amsdu::AMSDUSubframe;
    use mac_parser::ZERO;
    let _data_frame = DataFrameBuilder::new()
        .to_and_from_ds()
        .category_qos()
        .payload_amsdu::<&[AMSDUSubframe<&[u8]>]>(&[])
        .receiver_address(ZERO)
        .transmitter_address(ZERO)
        .bssid(ZERO)
        .build();
}
