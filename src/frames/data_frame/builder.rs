use core::marker::PhantomData;

use mac_parser::MACAddress;

use crate::{common::FragSeqInfo, frame_control_field::FCFFlags, type_state::*};

use self::type_state::{
    Data, DataFrameCategory, DataNull, HasPayload, Payload, QoS,
    QoSNull,
};

use super::{amsdu::AMSDUPayload, DataFrame, DataFramePayload, DataFrameSubtype};

pub mod type_state {
    use scroll::ctx::{MeasureWith, TryIntoCtx};

    use crate::frames::data_frame::AMSDUPayload;

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
    address_4: Option<MACAddress>,
    payload: Option<DataFramePayload<'a>>,
    _phantom: PhantomData<(DS, Category, PayloadType, Address4)>,
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
            address_4: None,
            payload: Some(DataFramePayload::Single(payload)),
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
            address_4: None,
            payload: Some(DataFramePayload::AMSDU(payload)),
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
            _phantom: PhantomData,
        }
    }
}
impl<'a, Category: DataFrameCategory, PayloadType, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, Address1, Address2, Address3, ()> {
    pub const fn destination_address(
        self,
        desination_address: MACAddress
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, MACAddress, Address2, Address3, ()> {
        DataFrameBuilderInner {
            address_1: desination_address,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
    pub const fn source_address(
        self,
        source_address: MACAddress
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, Address1, MACAddress, Address3, ()> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: source_address,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
    pub const fn bssid(
        self,
        bssid: MACAddress
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, MACAddress, Address2, Address3, ()> {
        DataFrameBuilderInner {
            address_1: bssid,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, PayloadType, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, FromDS, Category, PayloadType, Address1, Address2, Address3, ()> {
    pub const fn destination_address(
        self,
        desination_address: MACAddress
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, MACAddress, Address2, Address3, ()> {
        DataFrameBuilderInner {
            address_1: desination_address,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, Address2, Address3, ()> {
    pub const fn source_address(
        self,
        source_address: MACAddress
    ) -> DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, Address2, MACAddress, ()> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: source_address,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
    pub const fn bssid(
        self,
        bssid: MACAddress
    ) -> DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, MACAddress, Address3, ()> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: bssid,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, FromDS, Category, AMSDUPayload<'a>, Address1, Address2, Address3, ()> {
    pub const fn bssid(
        self,
        bssid: MACAddress
    ) -> DataFrameBuilderInner<'a, FromDS, Category, &'a [u8], Address1, MACAddress, MACAddress, ()> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: bssid,
            address_3: bssid,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, PayloadType, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, ToDS, Category, PayloadType, Address1, Address2, Address3, ()> {
    pub const fn source_address(
        self,
        source_address: MACAddress
    ) -> DataFrameBuilderInner<'a, NeitherToNorFromDS, Category, PayloadType, Address1, MACAddress, Address3, ()> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: source_address,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, Address3, ()> {
    pub const fn desination_address(
        self,
        desination_address: MACAddress
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], Address1, Address2, MACAddress, ()> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: desination_address,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
    pub const fn bssid(
        self,
        bssid: MACAddress
    ) -> DataFrameBuilderInner<'a, ToDS, Category, &'a [u8], MACAddress, Address2, Address3, ()> {
        DataFrameBuilderInner {
            address_1: bssid,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy> DataFrameBuilderInner<'a, ToDS, Category, AMSDUPayload<'a>, Address1, Address2, Address3, ()> {
    pub const fn bssid(
        self,
        bssid: MACAddress
    ) -> DataFrameBuilderInner<'a, ToDS, Category, AMSDUPayload<'a>, MACAddress, Address2, MACAddress, ()> {
        DataFrameBuilderInner {
            address_1: bssid,
            address_2: self.address_2,
            address_3: bssid,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy, Address4: Copy> DataFrameBuilderInner<'a, ToAndFromDS, Category, &'a [u8], Address1, Address2, Address3, Address4> {
    pub const fn desination_address(
        self,
        desination_address: MACAddress
    ) -> DataFrameBuilderInner<'a, ToAndFromDS, Category, &'a [u8], Address1, Address2, MACAddress, Address4> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: desination_address,
            address_4: self.address_4,
            payload: self.payload,
            _phantom: PhantomData
        }
    }
    pub const fn source_address(
        self,
        source_address: MACAddress
    ) -> DataFrameBuilderInner<'a, ToAndFromDS, Category, &'a [u8], Address1, Address2, Address3, MACAddress> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: Some(source_address),
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, Category: DataFrameCategory, Address1: Copy, Address2: Copy, Address3: Copy, Address4: Copy> DataFrameBuilderInner<'a, ToAndFromDS, Category, AMSDUPayload<'a>, Address1, Address2, Address3, Address4> {
    pub const fn bssid(
        self,
        bssid: MACAddress
    ) -> DataFrameBuilderInner<'a, ToAndFromDS, Category, AMSDUPayload<'a>, Address1, Address2, MACAddress, MACAddress> {
        DataFrameBuilderInner {
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: bssid,
            address_4: Some(bssid),
            payload: self.payload,
            _phantom: PhantomData
        }
    }
}
impl<'a, DS: DSField, Category: DataFrameCategory, PayloadType: Payload>
    DataFrameBuilderInner<'a, DS, Category, PayloadType, MACAddress, MACAddress, MACAddress, ()>
{
    #[inline]
    pub const fn build(self) -> DataFrame<'a> {
        DataFrame {
            sub_type: DataFrameSubtype::from_representation(Category::UPPER_TWO_BITS << 2),
            address_1: self.address_1,
            address_2: self.address_2,
            address_3: self.address_3,
            address_4: None,
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
