use octoscript_ui_l0::{InstanceStore, ValueOrigin};

#[test]
fn snapshot_roundtrip_keeps_cells_and_origins() {
    let mut state = InstanceStore::default();
    state.set_cell_with_origin("@card", "note", "hello".into(), ValueOrigin::UserInput);
    let bytes = state.snapshot_bytes().unwrap();
    let restored = InstanceStore::from_snapshot_bytes(&bytes).unwrap();
    assert_eq!(restored.get("@card", "note").unwrap(), "hello");
    assert_eq!(
        restored.origin("@card", "note"),
        Some(ValueOrigin::UserInput)
    );
    assert!(InstanceStore::from_snapshot_bytes(b"{\"schema\":99}").is_err());
    assert!(InstanceStore::from_snapshot_bytes(&vec![b' '; 1_048_577]).is_err());
}
