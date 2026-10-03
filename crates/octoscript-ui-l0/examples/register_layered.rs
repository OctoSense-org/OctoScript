//! Register 17 finance-brief-style `sys.*` helpers and check a card that uses them.
//!
//! Usage: cargo run -p octoscript-ui-l0 --example register_layered -- <card>

use octoscript_ui_l0::{catalog, catalog::SysContract, check_ui_l0_named};

fn finance_brief_capabilities() -> Vec<SysContract> {
    vec![
        SysContract {
            name: "sys.news_sina".into(),
            fields: &["id", "title", "source", "ts", "summary"],
            schema: None,
        },
        SysContract {
            name: "sys.quote_tencent".into(),
            fields: &["ticker", "name", "last", "change", "pct", "open"],
            schema: None,
        },
        SysContract {
            name: "sys.quote_stooq".into(),
            fields: &["ticker", "last", "open", "high", "low", "volume"],
            schema: None,
        },
        SysContract {
            name: "sys.quote_hyperliquid".into(),
            fields: &["ticker", "last", "change", "pct", "volume"],
            schema: None,
        },
        SysContract {
            name: "sys.quote_frankfurter".into(),
            fields: &["ticker", "last"],
            schema: None,
        },
        SysContract {
            name: "sys.synth_candles".into(),
            fields: &["ts", "open", "high", "low", "close", "volume"],
            schema: None,
        },
        SysContract {
            name: "sys.research_list".into(),
            fields: &["title", "summary", "coverage", "as_of"],
            schema: None,
        },
        SysContract {
            name: "sys.research_detail".into(),
            fields: &["title", "summary", "evidence_title", "evidence_body"],
            schema: None,
        },
        SysContract {
            name: "sys.fav_list".into(),
            fields: &["id", "kind", "ref", "ts"],
            schema: None,
        },
        SysContract {
            name: "sys.fav_toggle".into(),
            fields: &["id", "kind", "ref"],
            schema: None,
        },
        SysContract {
            name: "sys.settings_load".into(),
            fields: &["freq", "lang", "theme"],
            schema: None,
        },
        SysContract {
            name: "sys.settings_save".into(),
            fields: &["freq", "lang", "theme"],
            schema: None,
        },
        SysContract {
            name: "sys.stream_subscribe".into(),
            fields: &["topic", "interval_ms"],
            schema: None,
        },
        SysContract {
            name: "sys.stream_unsubscribe".into(),
            fields: &["topic"],
            schema: None,
        },
        SysContract {
            name: "sys.stream_freq".into(),
            fields: &["topic", "interval_ms"],
            schema: None,
        },
        SysContract {
            name: "sys.stream_tick".into(),
            fields: &["topic", "ts", "data"],
            schema: None,
        },
        SysContract {
            name: "sys.datasource_status".into(),
            fields: &[
                "name",
                "kind",
                "status",
                "last_ok",
                "latency_ms",
                "last_err",
            ],
            schema: None,
        },
    ]
}

fn main() {
    let mut ok_count = 0;
    let mut err_count = 0;
    for cap in finance_brief_capabilities() {
        match catalog::register(cap) {
            Ok(_) => ok_count += 1,
            Err(e) => {
                eprintln!("register failed: {e}");
                err_count += 1;
            }
        }
    }
    eprintln!(
        "registered: {ok_count} ok, {err_count} err; names = {:?}",
        catalog::registered_names()
    );
    let path = std::env::args()
        .nth(1)
        .expect("usage: register_layered <card>");
    let card = std::fs::read_to_string(&path).expect("read");
    let r = check_ui_l0_named("register_layered_example", &card);
    println!("  valid = {}  level = {:?}", r.valid, r.level);
    for d in &r.diagnostics {
        println!("  {}:{} {}", d.line, d.column, d.message);
    }
    if err_count > 0 {
        std::process::exit(1);
    }
}
