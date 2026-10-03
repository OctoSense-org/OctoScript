#![no_main]

//! Fuzz the runtime-registered catalog layer.
//!
//! The build-time `catalog::ANSWERS` table is closed; `catalog::register`
//! layers host-owned helpers on top. This target exercises the boundary:
//! every iteration picks a (name, fields) pair from a small fixed pool and
//! calls `register` (allowing `Err` from duplicate registrations) and the
//! three lookup functions. The expectation is "never panic, never UB".

use libfuzzer_sys::fuzz_target;
use octoscript_ui_l0::catalog::{self, SysContract};

const FAKE_NAMES: &[&str] = &[
    "sys.fuzz_a",
    "sys.fuzz_b",
    "sys.fuzz_c",
    "sys.fuzz_d",
    "sys.fuzz_e",
    "sys.fuzz_f",
    "sys.fuzz_g",
    "sys.fuzz_h",
];

const FIELD_SETS: &[&[&str]] = &[
    &[],
    &["a"],
    &["a", "b"],
    &["a", "b", "c"],
    &["id", "title", "ts", "key", "summary"],
];

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let name = FAKE_NAMES[(data[0] as usize) % FAKE_NAMES.len()];
    let fields = FIELD_SETS[((data[0] as usize) >> 4) % FIELD_SETS.len()];

    // Register: Ok on first attempt, Err on duplicate. Both are valid.
    let _ = catalog::register(SysContract {
        name: name.to_string(),
        fields,
        schema: None,
    });

    // Lookups must not panic regardless of whether `name` is registered.
    let _ = catalog::answers(name);
    let _ = catalog::registered_names();
    let _ = catalog::aggregates(name);
});
