//! `sys.digest`: a host-resolved digest source (OctoSense ADR 0002 §7).
use octoscript_ui_l0::{
    catalog, check_ui_l0_named, makepad, realize, source_plan, NodeValue, SourceArg, SourceBinding,
    UiNode,
};
use serde_json::json;

const CARD: &str = include_str!("fixtures/digest.card");

fn texts(node: &UiNode, out: &mut Vec<String>) {
    for (_, v) in &node.args {
        if let NodeValue::Text(t) = v {
            out.push(t.clone());
        }
    }
    for c in &node.children {
        texts(c, out);
    }
}

fn errors(card: &str) -> Vec<String> {
    check_ui_l0_named("digest", card)
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn a_digest_card_is_l0_and_the_host_resolves_it() {
    let report = check_ui_l0_named("digest", CARD);
    assert!(report.valid, "{:?}", report.diagnostics);
    assert_eq!(report.level, octoscript_ui_l0::Level::L0);
    let plan = source_plan(CARD);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    let request = &plan.requests[0];
    assert_eq!(
        (request.name.as_str(), request.helper.as_str()),
        ("brief", "sys.digest")
    );
    assert!(request
        .args
        .contains(&("app".into(), SourceArg::Text("os.news".into()))));
    assert!(request
        .args
        .contains(&("id".into(), SourceArg::Text("morning".into()))));
}

#[test]
fn the_card_shows_only_what_the_digest_holds() {
    let data = json!({"brief": {
        "status": "ready", "topic": "city infrastructure", "language": "en",
        "summary": "Three cities invest in transit and water.", "retrieved_at": "2026-09-20T08:00:00Z",
        "count": 1,
        "points": [{"id": "p1", "text": "Harbor City orders 120 electric buses.", "label": "", "cite": "1", "citations": [0]}],
        "sources": [{"id": "s1", "n": "1", "title": "Harbor City approves electric bus order", "source": "Harbor Daily", "url": "https://example.invalid/buses", "published_at": "2026-09-19T10:00:00Z"}]
    }});
    let report = realize(CARD, &data, Default::default());
    let root = report.complete_root().expect("realizes");
    let mut out = Vec::new();
    texts(root, &mut out);
    for want in [
        "Three cities invest in transit and water.",
        "Harbor City orders 120 electric buses.",
        "Harbor Daily",
        "city infrastructure",
    ] {
        assert!(out.iter().any(|t| t == want), "{want:?} not in {out:?}");
    }
    assert!(!out.iter().any(|t| t == "No digest yet"), "{out:?}");

    // A missing digest: an empty record the host marks failed. The card says
    // so, from its own vocabulary, and states nothing else.
    let data = json!({"brief": {"status": "missing", "points": [], "sources": []}, "$status": {"brief": "failed"}});
    let report = realize(CARD, &data, Default::default());
    let mut out = Vec::new();
    texts(report.complete_root().expect("realizes"), &mut out);
    assert!(out.iter().any(|t| t == "No digest yet"), "{out:?}");
}

#[test]
fn app_and_id_are_required_and_checked() {
    let with = |args: &str| CARD.replace("app: \"os.news\", id: \"morning\",", args);
    assert!(errors(&with("id: \"morning\","))
        .iter()
        .any(|e| e.contains("needs `app`")));
    assert!(errors(&with("app: \"os.news\","))
        .iter()
        .any(|e| e.contains("needs `id`")));
    // A traversal is not an id; neither is an app id with a slash.
    assert!(errors(&with("app: \"os.news\", id: \"../mail/x\","))
        .iter()
        .any(|e| e.contains("not a digest id")));
    assert!(errors(&with("app: \"os/news\", id: \"morning\","))
        .iter()
        .any(|e| e.contains("not an app id")));
    // `app` is never computed: the host compares it with the publisher.
    let card = with("app: state.owner, id: \"morning\",").replace(
        "copy label",
        "state owner { shape: text, initial: \"os.news\" }\ncopy label",
    );
    assert!(
        errors(&card)
            .iter()
            .any(|e| e.contains("must be a literal app id")),
        "{:?}",
        errors(&card)
    );
    // `id` may come from state.
    let card = with("app: \"os.news\", id: state.run,").replace(
        "copy label",
        "state run { shape: text, initial: \"morning\" }\ncopy label",
    );
    assert!(errors(&card).is_empty(), "{:?}", errors(&card));
    // Only what the digest answers, and no other argument.
    assert!(!errors(&CARD.replace("fields: [topic,", "fields: [password, topic,")).is_empty());
    assert!(!errors(&with("app: \"os.news\", id: \"morning\", path: \"/etc\",")).is_empty());
}

#[test]
fn the_digest_is_read_only_and_has_no_backend_call() {
    assert!(catalog::mutable("sys.digest").is_none());
    assert!(catalog::answers("sys.digest").unwrap().contains(&"sources"));
    let binding = SourceBinding {
        helper: "sys.digest".into(),
        field: "summary".into(),
        nested: vec![],
        args: vec![
            ("app".into(), "os.news".into()),
            ("id".into(), "morning".into()),
        ],
    };
    assert!(makepad::vm_call(&binding).is_none());
}
