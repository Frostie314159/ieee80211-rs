pub mod action;

mod beacon;
use action::{ActionBody, RawActionBody};
pub use beacon::{BeaconBody, BeaconLikeBody, BeaconSubtype, ProbeResponseSubtype};

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
    #[doc(hidden)]
    /// If the frame is an action frame, this will check, wether the supplied [ReadActionBody] matches itself.
    ///
    /// This has to be implemented for all frame types, due to the [match_frames](crate::match_frames) macro, and is meant for internal use.
    /// For all non-action management frames, this will always return false.
    fn read_action_body_matches(_action_body: RawActionBody<'_>) -> bool {
        false
    }
}

/// A trait indicating, that the management frame body has elements.
pub trait HasElements<ElementContainer> {
    /// Get the elements from the frame body.
    fn get_elements(&self) -> &ElementContainer;
}
macro_rules! mgmt_frame_bodies_with_elements {
    (
        $(
            $frame_body_type:ident => $subtype:ident
        ),*
    ) => {
        $(
            impl<'a, ElementContainer> ManagementFrameBody for $frame_body_type<'a, ElementContainer> {
                const SUBTYPE: ManagementFrameSubtype = ManagementFrameSubtype::$subtype;
            }
            impl<ElementContainer> HasElements<ElementContainer> for $frame_body_type<'_, ElementContainer> {
                fn get_elements(&self) -> &ElementContainer {
                    &self.elements
                }
            }
        )*
    };
}
mgmt_frame_bodies_with_elements! {
    AssociationRequestBody => AssociationRequest,
    AssociationResponseBody => AssociationResponse,
    ProbeRequestBody => ProbeRequest,
    ProbeResponseBody => ProbeResponse,
    BeaconBody => Beacon,
    DisassociationBody => Disassociation,
    AuthenticationBody => Authentication,
    DeauthenticationBody => Deauthentication
}
impl ManagementFrameBody for RawActionBody<'_> {
    const SUBTYPE: ManagementFrameSubtype = ManagementFrameSubtype::Action;
    fn read_action_body_matches(_action_body: RawActionBody<'_>) -> bool {
        true
    }
}
impl<Body: ActionBody> ManagementFrameBody for Body {
    const SUBTYPE: ManagementFrameSubtype = ManagementFrameSubtype::Action;
    fn read_action_body_matches(action_body: RawActionBody<'_>) -> bool {
        Body::matches(action_body)
    }
}
