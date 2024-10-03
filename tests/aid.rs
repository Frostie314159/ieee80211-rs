use ieee80211::common::AssociationID;

#[test]
fn test_aid() {
    assert!(AssociationID::new_checked(1).is_some());
    assert!(AssociationID::new_checked(2007).is_some());
    assert!(AssociationID::new_checked(2007 | 0xc000).is_some());
    assert!(AssociationID::new_checked(2008).is_none());
    assert_eq!(AssociationID::new_checked(1).unwrap().into_bits(), 0xc001);
}
