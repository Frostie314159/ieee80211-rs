mod action;
pub use action::ActionBody;

mod beacon;
pub use beacon::{BeaconBody, BeaconLikeFrameBody, BeaconSubtype, ProbeResponseSubtype};

mod disassoc;
pub use disassoc::DisassociationBody;

mod probe;
pub use probe::{ProbeRequestBody, ProbeResponseBody};

mod assoc;
pub use assoc::{AssociationRequestBody, AssociationResponseBody};

mod deauth;
pub use deauth::DeauthenticationBody;

mod auth;
pub use auth::AuthenticationBody;

use crate::common::ManagementFrameSubtype;

/// A trait implemented by all management frames bodies.
pub trait ManagementFrameBody {
    const SUBTYPE: ManagementFrameSubtype;
}
macro_rules! mgmt_frame_bodies {
    (
        $(
            $frame_body_type:ident => $subtype:ident
        ),*
    ) => {
        $(
            impl<'a, ElementContainer> ManagementFrameBody for $frame_body_type<'a, ElementContainer> {
                const SUBTYPE: ManagementFrameSubtype = ManagementFrameSubtype::$subtype;
            }
        )*
    };
}
mgmt_frame_bodies! {
    AssociationRequestBody => AssociationRequest,
    AssociationResponseBody => AssociationResponse,
    ProbeRequestBody => ProbeRequest,
    ProbeResponseBody => ProbeResponse,
    BeaconBody => Beacon,
    DisassociationBody => Disassociation,
    AuthenticationBody => Authentication,
    DeauthenticationBody => Deauthentication,
    ActionBody => Action
}

/// A trait indicating, that the management frame body has elements.
pub trait HasElements<ElementContainer> {
    /// Get the elements from the frame body.
    fn get_elements(&self) -> &ElementContainer;
}
macro_rules! has_elements {
    (
        $(
            $frame_body_type:ident
        ),*
    ) => {
        $(
            impl<ElementContainer> HasElements<ElementContainer> for $frame_body_type<'_, ElementContainer> {
                fn get_elements(&self) -> &ElementContainer {
                    &self.elements
                }
            }
        )*
    };
}
has_elements! {
    AssociationRequestBody,
    AssociationResponseBody,
    ProbeRequestBody,
    ProbeResponseBody,
    BeaconBody,
    DisassociationBody,
    AuthenticationBody,
    DeauthenticationBody
}
