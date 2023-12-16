use alloc::borrow::Cow;
use mac_parser::MACAddress;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::FragSeqInfo;

serializable_enum! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub enum CategoryCode: u8 {
        #[default]
        VendorSpecific => 127
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CategoryBody<'a> {
    VendorSpecific {
        oui: [u8; 3],
        payload: Cow<'a, [u8]>,
    },
}
impl MeasureWith<()> for CategoryBody<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        4 + match self {
            CategoryBody::VendorSpecific { payload, .. } => payload.len(),
        }
    }
}
impl<'a> TryFromCtx<'a> for CategoryBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let category_code = CategoryCode::from_representation(from.gread(&mut offset)?);
        Ok((
            match category_code {
                CategoryCode::VendorSpecific => {
                    let mut oui = [0x00; 3];
                    oui.copy_from_slice(from.gread_with(&mut offset, 3)?);
                    let slice_len = from.len() - offset;
                    let payload = from.gread_with::<&'_ [u8]>(&mut offset, slice_len)?.into();
                    CategoryBody::VendorSpecific { oui, payload }
                }
                _ => {
                    return Err(scroll::Error::BadInput {
                        size: category_code.to_representation() as usize,
                        msg: "Category code not yet implented.",
                    })
                }
            },
            offset,
        ))
    }
}
impl<'a> TryIntoCtx for CategoryBody<'a> {
    type Error = scroll::Error;
    fn try_into_ctx(self, data: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        match self {
            Self::VendorSpecific { oui, payload } => {
                data.gwrite(
                    CategoryCode::VendorSpecific.to_representation(),
                    &mut offset,
                )?;
                data.gwrite(oui.as_slice(), &mut offset)?;
                data.gwrite::<&'_ [u8]>(&payload, &mut offset)?;
            }
        }
        Ok(offset)
    }
}

pub struct ActionFrame<'a> {
    pub duration: u16,
    pub destination: MACAddress,
    pub source: MACAddress,
    pub bssid: MACAddress,
    pub frag_seq_info: FragSeqInfo,
    pub body: CategoryBody<'a>,
}
impl MeasureWith<()> for ActionFrame<'_> {
    fn measure_with(&self, ctx: &()) -> usize {
        22 + self.body.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for ActionFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let duration = from.gread_with(&mut offset, Endian::Little)?;
        let destination = MACAddress::new(from.gread_with::<[u8; 6]>(&mut offset, ctx)?);
        let source = MACAddress::new(from.gread_with::<[u8; 6]>(&mut offset, ctx)?);
        let bssid = MACAddress::new(from.gread_with::<[u8; 6]>(&mut offset, ctx)?);
        let frag_seq_info = FragSeqInfo::from_representation(from.gread(&mut offset)?);
        let body = from.gread(&mut offset)?;
        Ok((
            Self {
                duration,
                destination,
                source,
                bssid,
                frag_seq_info,
                body,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for ActionFrame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.duration, &mut offset)?;
        buf.gwrite(self.destination, &mut offset)?;
        buf.gwrite(self.source, &mut offset)?;
        buf.gwrite(self.bssid, &mut offset)?;
        buf.gwrite(self.frag_seq_info.to_representation(), &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;

        Ok(offset)
    }
}
