//! Tests for the runtime catalog registration API added to `catalog`.
//!
//! Every test uses a name that is unique to itself, so parallel test
//! execution cannot cause one test's `register` to collide with another's.

use octoscript_ui_l0::{catalog::*, check_ui_l0};

#[test]
fn register_then_lookup_returns_fields() {
    let contract = SysContract {
        name: "sys.fake_a".into(),
        fields: &["a", "b"],
        schema: None,
    };
    register(contract.clone()).expect("register ok");
    assert_eq!(answers("sys.fake_a"), Some(&["a", "b"][..]));
}

#[test]
fn register_rejects_collision_with_answers() {
    let contract = SysContract {
        name: "sys.geocode".into(),
        fields: &["x"],
        schema: None,
    };
    let err = register(contract).unwrap_err();
    assert!(err.contains("ANSWERS"), "got: {err}");
}

#[test]
fn register_rejects_collision_with_aggregates() {
    let contract = SysContract {
        name: "sys.series".into(),
        fields: &["x"],
        schema: None,
    };
    let err = register(contract).unwrap_err();
    assert!(err.contains("AGGREGATES"), "got: {err}");
}

#[test]
fn register_rejects_collision_with_mutable() {
    let contract = SysContract {
        name: "sys.cities".into(),
        fields: &["x"],
        schema: None,
    };
    let err = register(contract).unwrap_err();
    assert!(err.contains("MUTABLE"), "got: {err}");
}

#[test]
fn register_rejects_duplicate_name() {
    let contract = SysContract {
        name: "sys.fake_b".into(),
        fields: &["a"],
        schema: None,
    };
    // The first register may already have been done by an earlier parallel
    // run of this same test (e.g. `cargo test` re-runs after a fix). Either
    // outcome is fine: what we are asserting is that the SECOND register
    // returns the duplicate-name error.
    let _ = register(contract.clone());
    let err = register(contract).unwrap_err();
    assert!(err.contains("already"), "got: {err}");
}

#[test]
fn register_rejects_non_sys_prefix() {
    let contract = SysContract {
        name: "foo.bar".into(),
        fields: &["a"],
        schema: None,
    };
    let err = register(contract).unwrap_err();
    assert!(err.contains("sys."), "got: {err}");
}

#[test]
fn register_accepts_multiple_distinct_names() {
    for name in ["sys.fake_c1", "sys.fake_c2", "sys.fake_c3"] {
        let _ = register(SysContract {
            name: name.into(),
            fields: &["a"],
            schema: None,
        });
    }
    let listed = registered_names();
    assert!(listed.iter().any(|n| n == "sys.fake_c1"));
    assert!(listed.iter().any(|n| n == "sys.fake_c2"));
    assert!(listed.iter().any(|n| n == "sys.fake_c3"));
}

#[test]
fn validate_sources_accepts_registered_helper() {
    let _ = register(SysContract {
        name: "sys.fake_c_accept".into(),
        fields: &["x", "y"],
        schema: None,
    });
    let card = "source n sys.fake_c_accept(fields: [x])\nview root Surface { for v in n key v.x { TextRow(text: v.x) } }";
    let r = check_ui_l0(card);
    assert!(r.valid, "diagnostics: {:?}", r.diagnostics);
}

#[test]
fn validate_sources_rejects_unknown_field_for_registered() {
    let _ = register(SysContract {
        name: "sys.fake_c_reject".into(),
        fields: &["x", "y"],
        schema: None,
    });
    let card = "source n sys.fake_c_reject(fields: [bogus])\nview root Surface { for v in n key v.bogus { TextRow(text: v.bogus) } }";
    let r = check_ui_l0(card);
    assert!(
        !r.valid,
        "expected invalid, diagnostics: {:?}",
        r.diagnostics
    );
}

#[test]
fn register_and_concurrent_lookup_no_ub_under_miri() {
    use std::thread;
    let _ = register(SysContract {
        name: "sys.miri_test".into(),
        fields: &["a", "b", "c"],
        schema: None,
    });
    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..1000 {
                    let _ = answers("sys.miri_test");
                    let _ = registered_names();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}
