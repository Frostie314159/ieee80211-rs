use probe::ProbeResponeBody;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::elements::ReadElements;

mod action;
pub use action::ActionFrameBody;

mod beacon;
pub use beacon::{BeaconFrameBody, BeaconLikeFrameBody, BeaconSubtype, ProbeResponseSubtype};

mod disassoc;
pub use disassoc::DisassociationFrameBody;

mod probe;
pub use probe::ProbeRequestBody;

mod assoc;
pub use assoc::{AssociationRequestBody, AssociationResponseBody};

mod deauth;
pub use deauth::DeauthenticationBody;

mod auth;
pub use auth::AuthenticationBody;

macro_rules! management_frame_bodies {
    (
        $(
            #[$enum_meta:meta]
        )*
        pub enum $enum_name:ident<$lt:lifetime, $($enum_generic:ident $(: $($enum_generic_bound:path),*)? = $enum_generic_default:ty),*> {
            $(
                $body_name:ident : $body_subtype:expr $(=> $body_type:ident$(<$($body_type_generic:tt),*>)?)?
            ),*
        }
    ) => {
        $(
            #[$enum_meta]
        )*
        pub enum $enum_name<$lt, $($enum_generic = $enum_generic_default),*>
        where
            $(
                $($enum_generic: $($enum_generic_bound + )*)?
            ),*
        {
            $(
                $body_name $(($body_type$(<$($body_type_generic),*>)?))?,
            )*
            Unknown {
                subtype: ManagementFrameSubtype,
                body: &$lt [u8]
            }
        }
        crate::macro_bits::serializable_enum! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            /// The subtype of a management frame.
            pub enum ManagementFrameSubtype: u8 {
                $(
                    $body_name => $body_subtype
                ),*
            }
        }
        impl $enum_name<'_> {
            pub const fn length_in_bytes(&self) -> usize {
                match self {
                    $(
                        Self::$body_name$((frame) if { let _ = stringify!($body_type); true })? => 0 $(+ {
                            let _ = stringify!($body_type);
                            frame.length_in_bytes()
                        })?,
                    )*
                    Self::Unknown { body, ..} => body.len(),
                    _ => unreachable!()
                }
            }
        }
        impl<$lt, $($enum_generic $(: $($enum_generic_bound + )*)?),*> $enum_name<$lt, $($enum_generic),*> {
            /// This returns the subtype of the body.
            /// It's mostly for internal use, but since it might be useful it's `pub`.
            pub const fn get_subtype(&self) -> ManagementFrameSubtype {
                match self {
                    $(
                        Self::$body_name$((_) if { let _ = stringify!($body_type); true})? => ManagementFrameSubtype::$body_name,
                    )*
                    Self::Unknown {
                        subtype,
                        ..
                    } => *subtype,
                    _ => unreachable!()
                }
            }
        }
        impl<$lt, $($enum_generic $(: $($enum_generic_bound + )* $lt)?),*> MeasureWith<()> for $enum_name<$lt, $($enum_generic),*> {
            fn measure_with(&self, ctx: &()) -> usize {
                match self {
                    $(
                        Self::$body_name$((frame) if { let _ = stringify!($body_type); true })? => 0 $(+ {
                            let _ = stringify!($body_type);
                            frame.measure_with(ctx)
                        })?,
                    )*
                    Self::Unknown { body, ..} => body.len(),
                    _ => unreachable!()
                }
            }
        }
        impl<$lt> TryFromCtx<$lt, ManagementFrameSubtype> for $enum_name<$lt> {
            type Error = scroll::Error;
            fn try_from_ctx(
                from: &$lt [u8],
                sub_type: ManagementFrameSubtype,
            ) -> Result<(Self, usize), Self::Error> {
                let mut offset = 0;
                Ok((
                    match sub_type {
                        $(
                            ManagementFrameSubtype::$body_name => Self::$body_name$(({
                                let _ = stringify!($body_type);
                                from.gread(&mut offset)?
                            }))?,
                        )*
                        _ => {
                            return Err(scroll::Error::BadInput {
                                size: offset,
                                msg: "Management frame subtype not implemented.",
                            })
                        }
                    },
                    offset,
                ))
            }
        }
        impl<$lt, $($enum_generic $(: $($enum_generic_bound + )* $lt)?),*> TryIntoCtx for $enum_name<$lt, $($enum_generic),*> {
            type Error = scroll::Error;
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
                let mut offset = 0;

                match self {
                    $(
                        Self::$body_name$((body) if { let _ = stringify!($body_type); true})? => {
                            $(
                                let _ = stringify!($body_type);
                                buf.gwrite(body, &mut offset)?;
                            )?
                        },
                    )*
                    Self::Unknown { body, .. } => {
                        buf.gwrite(body, &mut offset)?;
                    },
                    _ => unreachable!()
                }

                Ok(offset)
            }
        }
        pub trait ToManagementFrameBody<$lt, $($enum_generic = $enum_generic_default),*>
        where
        $(
            $enum_generic: $($(
                $enum_generic_bound +
            )*)?
        ),*
        {
            fn to_management_frame_body(self) -> $enum_name<$lt, $($enum_generic),*>;
        }
    };
}

management_frame_bodies! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// This is the body of a management frame.
    /// The rest of the frame can be found in [crate::frames::ManagementFrame].
    pub enum ManagementFrameBody<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error>, MeasureWith<()> = ReadElements<'a>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error>, MeasureWith<()> = &'a [u8]
    > {
        AssociationRequest: 0b0000 => AssociationRequestBody<'a, ElementContainer>,
        AssociationResponse: 0b0001 => AssociationResponseBody<'a, ElementContainer>,
        ProbeRequest: 0b0100 => ProbeRequestBody<'a, ElementContainer>,
        ProbeRespone: 0b0101 => ProbeResponeBody<'a, ElementContainer>,
        Beacon: 0b1000 => BeaconFrameBody<'a, ElementContainer>,
        ATIM: 0b1001,
        Disassociation: 0b1010 => DisassociationFrameBody<'a, ElementContainer>,
        Authentication: 0b1011 => DeauthenticationBody<'a, ElementContainer>,
        Deauthentication: 0b1100 => DeauthenticationBody<'a, ElementContainer>,
        Action: 0b1101 => ActionFrameBody<'a, ActionFramePayload>,
        ActionNoACK: 0b1110 => ActionFrameBody<'a, ActionFramePayload>
    }
}
