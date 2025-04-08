use ieee80211::{elements::MeshIDElement, mesh_id};

use crate::roundtrip_test;

// We can't test the [mesh_id] macro, since rust doesn't support expected build failures.
// This isn't doesn't really matter, since it's short enough to audit by hand.

const EXPECTED_MESHID_STRING: &str = "Test";
const EXPECTED_MESHID_ELEMENT: MeshIDElement<&str> = mesh_id!(EXPECTED_MESHID_STRING);
const EXPECTED_MESHID_ELEMENT_BYTES: &[u8] = EXPECTED_MESHID_STRING.as_bytes();
const WILDCARD_MESHID_ELEMENT: MeshIDElement<&str> = mesh_id!("");
// One byte too long.
const INVALID_MESHID_STRING: &str = "Lorem ipsum dolor sit amet augue.";

roundtrip_test!(
    test_mesh_id_element_rw,
    MeshIDElement,
    EXPECTED_MESHID_ELEMENT,
    EXPECTED_MESHID_ELEMENT_BYTES
);

#[test]
fn test_mesh_id_element_misc() {
    assert!(
        WILDCARD_MESHID_ELEMENT.is_hidden(),
        "Wildcard Mesh ID wasn't marked as hidden. How did this happen?"
    );
    // Not so fun fact: This test technically already caught an error, since I screwed up when writing the original function...
    assert_eq!(
        EXPECTED_MESHID_STRING.len(),
        EXPECTED_MESHID_ELEMENT.length_in_bytes(),
        "Length in bytes returned didn't match what was expected."
    );
    assert_eq!(
        MeshIDElement::new(EXPECTED_MESHID_STRING),
        Some(MeshIDElement::new_unchecked(EXPECTED_MESHID_STRING)),
        "Creating a Mesh ID element, with a valid Mesh ID failed."
    );
    assert!(
        MeshIDElement::new(INVALID_MESHID_STRING).is_none(),
        "Creating a Mesh ID element, with an invalid Mesh ID succeeded."
    );
}
