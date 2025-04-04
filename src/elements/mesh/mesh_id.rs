use core::{fmt::Display, marker::PhantomData};

use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use crate::elements::{Element, ElementID};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The Mesh ID element holds the human-readable identifier of a mesh BSS.
///
/// The mesh_id isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub struct MeshIDElement<'a, MeshID = &'a str> {
    mesh_id: MeshID,
    _phantom: PhantomData<&'a ()>,
}
impl<'a> MeshIDElement<'a> {
    /// Create a new Mesh ID element.
    ///
    /// This returns [None] if `mesh_id` is longer than 32 bytes.
    pub const fn const_new(mesh_id: &'a str) -> Option<Self> {
        if mesh_id.len() <= 32 {
            Some(Self {
                mesh_id,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
}
impl<MeshID: AsRef<str>> MeshIDElement<'_, MeshID> {
    /// Create a new Mesh ID element.
    ///
    /// This returns [None] if `mesh_id` is longer than 32 bytes.
    pub fn new(mesh_id: MeshID) -> Option<Self> {
        if mesh_id.as_ref().len() <= 32 {
            Some(Self {
                mesh_id,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
    #[doc(hidden)]
    #[inline]
    // Only for internal use, by macros.
    pub const fn new_unchecked(mesh_id: MeshID) -> Self {
        Self {
            mesh_id,
            _phantom: PhantomData,
        }
    }

    #[inline]
    /// Get the mesh ID as a [str] reference.
    pub fn mesh_id(&self) -> &str {
        self.mesh_id.as_ref()
    }

    #[inline]
    /// Take the Mesh ID.
    pub fn take_mesh_id(self) -> MeshID {
        self.mesh_id
    }

    /// Check if the Mesh ID is empty.
    ///
    /// # Returns
    /// - [`true`] If the Mesh ID is empty.
    /// - [`false`] If the Mesh ID isn't empty.
    pub fn is_empty(&self) -> bool {
        self.mesh_id().is_empty()
    }
    /// Return the length in bytes.
    ///
    /// This is useful for hardcoded Mesh IDs, since it's `const`.
    pub fn length_in_bytes(&self) -> usize {
        self.mesh_id().len()
    }
}
impl<MeshID: AsRef<str>> Element for MeshIDElement<'_, MeshID> {
    const ELEMENT_ID: ElementID = ElementID::Id(114);
    type ReadType<'a> = MeshIDElement<'a>;
}
impl<MeshID: AsRef<str>> AsRef<str> for MeshIDElement<'_, MeshID> {
    fn as_ref(&self) -> &str {
        self.mesh_id()
    }
}
impl<MeshID: AsRef<str>> Display for MeshIDElement<'_, MeshID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.mesh_id.as_ref())
    }
}
#[cfg(feature = "defmt")]
impl<MeshID: AsRef<str>> defmt::Format for MeshIDElement<'_, MeshID> {
    fn format(&self, fmt: defmt::Formatter) {
        self.mesh_id.as_ref().format(fmt)
    }
}
impl<'a> TryFromCtx<'a> for MeshIDElement<'a> {
    type Error = scroll::Error;
    #[inline]
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 32 {
            return Err(scroll::Error::TooBig {
                size: 32,
                len: from.len(),
            });
        }
        <&'a str as TryFromCtx<'a, StrCtx>>::try_from_ctx(from, StrCtx::Length(from.len()))
            .map(|(mesh_id, len)| (Self::new_unchecked(mesh_id), len))
    }
}
impl<MeshID: AsRef<str>> MeasureWith<()> for MeshIDElement<'_, MeshID> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<MeshID: AsRef<str>> TryIntoCtx for MeshIDElement<'_, MeshID> {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.mesh_id(), 0)
    }
}
#[macro_export]
/// Generate an [MeshIDElement], while performing all validation at compile time.
///
/// This macro requires, that the passed parameter is either a literal or a const and must be a `&str`.
/// ```
/// use ieee80211::mesh_id;
///
/// let mesh_id_element = mesh_id!("meshtest");
/// assert_eq!(mesh_id_element.mesh_id(), "meshtest");
/// ```
/// If the provided literal is longer than 32 bytes, the macro will panic at compile time.
///
/// ```compile_fail
/// use ieee80211::mesh_id;
///
/// let _mesh_id_element = mesh_id!("Some unreasonably long MeshID, for whatever reason.");
/// ```
macro_rules! mesh_id {
    ($mesh_id:expr) => {{
        use ::ieee80211::elements::mesh::MeshIDElement;
        const RESULT: MeshIDElement<'static> = {
            ::core::assert!(
                $mesh_id.as_bytes().len() <= 32,
                "MeshIDs must not exceed a length of more than 32 bytes."
            );
            MeshIDElement::new_unchecked($mesh_id)
        };
        RESULT
    }};
}
