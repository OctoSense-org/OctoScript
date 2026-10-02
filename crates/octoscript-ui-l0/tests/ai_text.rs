//! §4.2 and §5.15: model-written text in text slots, marked AI-written; actions
//! stay strict; and an in-card chat bound to a host conversation.
use octoscript_ui_l0::*;
use serde_json::json;

const CHAT: &str = include_str!("fixtures/chat.card");

fn errors(card: &str) -> Vec<String> {
    check_ui_l0_named("ai", card)
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
}

fn root(card: &str, data: serde_json::Value) -> UiNode {
    let report = realize(card, &data, RealizeLimits::default());
    report.complete_root().expect("realizes").clone()
}

fn find<'a>(node: &'a UiNode, kind: &str, out: &mut Vec<&'a UiNode>) {
    if node.kind == kind {
        out.push(node);
    }
    for c in &node.children {
        find(c, kind, out);
    }
}

fn nodes<'a>(node: &'a UiNode, kind: &str) -> Vec<&'a UiNode> {
    let mut out = Vec::new();
    find(node, kind, &mut out);
    out
}

fn assert_refused(card: &str, why: &str) {
    let errs = errors(card);
    assert!(
        errs.iter().any(|e| e.contains("text the model wrote")),
        "{why}: expected a model-text refusal, got {errs:#?}\n{card}"
    );
}

// ─── model-written text in a text slot ───────────────────────────────────────

const SUMMARY: &str = r#"
copy title { class: vocabulary, en: "TODAY" }
copy gist  { class: model-copy, en: "Markets were calm; rates held." }
view root Surface {
  TextEyebrow(text: copy.title)
  TextBody(text: copy.gist)
}
"#;

/// A model-written summary renders, and every lowering marks it AI-written —
/// and marks nothing else.
#[test]
fn model_text_in_a_text_slot_renders_marked_ai_written() {
    let report = check_ui_l0_named("ai", SUMMARY);
    assert!(report.valid, "{:#?}", report.diagnostics);
    assert_eq!(report.level, Level::L0);

    let tree = root(SUMMARY, json!({}));
    let body = nodes(&tree, "TextBody")[0];
    let eyebrow = nodes(&tree, "TextEyebrow")[0];
    assert!(ai_written(body), "{body:#?}");
    assert!(!ai_written(eyebrow), "vocabulary is not AI-written");
    assert!(body
        .origins
        .contains(&("text".to_string(), ValueOrigin::Model)));

    let kit = kit::lower(&tree);
    assert_eq!(kit.matches("l0_ai_text(").count(), 1, "{kit}");
    assert!(
        kit.contains("l0_ai_text(l0_body(\"Markets were calm; rates held.\"))"),
        "{kit}"
    );
    let mp = makepad::lower(&tree);
    assert_eq!(mp.matches("l0_ai: true").count(), 1, "{mp}");
    let dsl = lower_dsl(&tree);
    assert_eq!(dsl.matches("ai: 1").count(), 1, "{dsl}");
}

const DIGEST: &str = r#"
source brief sys.digest(app: "os.news", id: "morning", fields: [topic, summary, points, id, text])
view root Surface {
  TextCaption(text: brief.topic)
  TextBody(text: brief.summary)
  for p in brief.points key p.id { TextRow(text: p.text) }
}
"#;

/// A host source's model-written fields carry the mark too, through a loop.
#[test]
fn a_digest_summary_and_its_points_are_marked_ai_written() {
    assert!(errors(DIGEST).is_empty(), "{:#?}", errors(DIGEST));
    let tree = root(
        DIGEST,
        json!({"brief": {"topic": "rates", "summary": "Rates held.",
            "points": [{"id": "p1", "text": "The bank held at 4%."}]}}),
    );
    assert!(ai_written(nodes(&tree, "TextBody")[0]));
    assert!(
        ai_written(nodes(&tree, "TextRow")[0]),
        "a point's text, via the loop"
    );
    assert!(
        !ai_written(nodes(&tree, "TextCaption")[0]),
        "the topic is not model text"
    );
}

/// A draft: model text written into a `text` state by a transition. The state
/// is model text from then on — rendered marked, and refused as an action's
/// input like the text itself.
#[test]
fn a_draft_holds_model_text_and_stays_marked() {
    const CARD: &str = r#"
copy suggestion { class: model-copy, en: "Thanks — I'll be there at 6." }
state draft { shape: text, initial: "" }
event use { draft: set(copy.suggestion) }
event edit { draft: set($value) }
view root Surface {
  Chip(text: "Use suggestion", on_tap: use)
  Field(text: draft, on_commit: edit, width: .fill)
  TextBody(text: draft)
}
"#;
    assert!(errors(CARD).is_empty(), "{:#?}", errors(CARD));
    let mut store = InstanceStore::default();
    let outcome = dispatch_reporting_with_origin(
        CARD,
        &mut store,
        "root",
        "use",
        None,
        &json!({}),
        ValueOrigin::UserInput,
    );
    assert!(
        outcome.applied,
        "set(copy.x) resolves from the card's own copy"
    );
    assert_eq!(
        store.get(CARD_STATE_KEY, "draft").and_then(|v| v.as_str()),
        Some("Thanks — I'll be there at 6.")
    );
    assert_eq!(
        store.origin(CARD_STATE_KEY, "draft"),
        Some(ValueOrigin::Model)
    );

    let tree = realize_with_state(CARD, &json!({}), &store, RealizeLimits::default())
        .complete_root()
        .unwrap()
        .clone();
    assert!(ai_written(nodes(&tree, "TextBody")[0]));
    assert!(
        ai_written(nodes(&tree, "Field")[0]),
        "the field shows the model's draft"
    );
    let kit = kit::lower(&tree);
    assert!(kit.contains("l0_ai_text(l0_field("), "{kit}");

    // What the user COMMITS from the field is theirs, and the mark goes.
    let field = nodes(&tree, "Field")[0];
    let origin = event_payload_origin(&tree, &field.key, "edit").unwrap();
    assert_eq!(origin, ValueOrigin::UserInput);
    dispatch_reporting_with_origin(
        CARD,
        &mut store,
        &field.key,
        "edit",
        Some(&json!("See you at 6")),
        &json!({}),
        origin,
    );
    let tree = realize_with_state(CARD, &json!({}), &store, RealizeLimits::default())
        .complete_root()
        .unwrap()
        .clone();
    assert!(!ai_written(nodes(&tree, "TextBody")[0]));
}

// ─── actions stay strict ─────────────────────────────────────────────────────

/// Every position where model text would be part of an action, a query, an
/// identity or a control is refused, whatever the text says.
#[test]
fn model_text_never_reaches_an_action_a_query_or_a_control() {
    let copy = r#"copy m { class: model-copy, en: "NVDA" }
source watch sys.watchlist(fields: [ticker])
source link sys.link(fields: [url])
source brief sys.digest(app: "os.news", id: "morning", fields: [summary, points, id, text])
state pick { shape: text, initial: "" }
state mode { shape: enum[a, b], initial: a }
event add { watch: append($value) }
event go { pick: set($value) }
"#;
    let cases: &[(&str, &str)] = &[
        (
            "a tap payload",
            "view root Row(on_tap: add, value: copy.m) { Rule() }",
        ),
        (
            "a chip payload",
            "view root Chip(text: \"Add\", on_tap: add, value: brief.summary)",
        ),
        (
            "a card payload",
            "view root Card(on_tap: go, value: copy.m) { Rule() }",
        ),
        (
            "a hero's payload",
            "view root TextHero(text: copy.m, on_tap: add, value: copy.m)",
        ),
        (
            "a chip's label",
            "view root Chip(text: copy.m, on_tap: add)",
        ),
        (
            "a tile's label",
            "view root Tile(label: brief.summary, value: pick)",
        ),
        (
            "a tab's label",
            "view root TabBar { Tab(icon: .home, label: copy.m, active: .on) }",
        ),
        ("a kit component", "view root Kit(component: copy.m)"),
        (
            "a field placeholder",
            "view root Field(text: pick, placeholder: copy.m, on_commit: go)",
        ),
        (
            "a guard",
            "view root Surface { when brief.summary == \"x\" { Rule() } }",
        ),
        (
            "a guard on model copy",
            "view root Surface { when pick == copy.m { Rule() } }",
        ),
        (
            "a loop key",
            "view root Surface { for p in brief.points key p.text { TextRow(text: p.text) } }",
        ),
        (
            "a component prop",
            "component C(t: text) { view TextBody(text: t) }\nview root C(t: copy.m)",
        ),
        ("an image source", "view root Photo(src: brief.summary)"),
    ];
    for (why, view) in cases {
        assert_refused(&format!("{copy}{view}"), why);
    }

    // A source argument: model text would choose what the host fetches.
    assert_refused(
        &format!(
            "{copy}source w sys.wiki(query: brief.summary)\nview root TextBody(text: w.extract)"
        ),
        "a source argument",
    );
    // A host store: model text would become a URL or reference an action reads.
    assert_refused(
        &format!(
            "{copy}event save {{ link: set(copy.m) }}\nview root Row(on_tap: save) {{ Rule() }}"
        ),
        "a write to sys.link",
    );
    // A non-text state: model text would choose a member.
    assert_refused(
        &format!(
            "{copy}event pickm {{ mode: set(copy.m) }}\nview root Row(on_tap: pickm) {{ Rule() }}"
        ),
        "a write into an enum",
    );
    // An initial: a draft is written by a transition the user triggers.
    assert_refused(
        "copy m { class: model-copy, en: \"x\" }\nstate s { shape: text, initial: copy.m }\n\
         view root TextBody(text: s)",
        "a state initial",
    );
}

/// Realization is gated on the checker: a card that puts model text in a
/// payload yields no tree at all, so no lowering can emit it as a target.
#[test]
fn a_card_with_misplaced_model_text_does_not_realize() {
    const CARD: &str = r#"
copy m { class: model-copy, en: "NVDA" }
source watch sys.watchlist(fields: [ticker])
event add { watch: append($value) }
view root Row(on_tap: add, value: copy.m) { TextBody(text: copy.m) }
"#;
    assert!(!errors(CARD).is_empty(), "the checker refuses it");
    let report = realize(CARD, &json!({}), RealizeLimits::default());
    assert!(report.root.is_none() && report.complete_root().is_err());
}

/// Taint follows a draft: once model text is written into a state, that state
/// cannot become a payload, a query or a second state's member either.
#[test]
fn a_draft_cannot_launder_model_text_into_an_action() {
    let base = r#"copy m { class: model-copy, en: "AAPL" }
source watch sys.watchlist(fields: [ticker])
state draft { shape: text, initial: "" }
state other { shape: text, initial: "" }
event fill { draft: set(copy.m) }
event relay { other: set(draft) }
event add { watch: append($value) }
"#;
    assert_refused(
        &format!("{base}view root Row(on_tap: add, value: draft) {{ Rule() }}"),
        "a draft as a tap payload",
    );
    assert_refused(
        &format!("{base}view root Row(on_tap: add, value: other) {{ Rule() }}"),
        "a draft copied to a second state, as a tap payload",
    );
    assert_refused(
        &format!(
            "{base}source q sys.wiki(query: state.draft)\nview root TextBody(text: q.extract)"
        ),
        "a draft as a source argument",
    );
    // Shown in a text slot, it is fine.
    let ok = format!("{base}view root Row(on_tap: fill) {{ TextBody(text: other) }}");
    assert!(errors(&ok).is_empty(), "{:#?}", errors(&ok));
}

/// Injection-style model text is DATA. Whatever it looks like — a tap target, a
/// DSL fragment, a link, markup — it lowers as one quoted literal in a text
/// slot, and the only targets in the output are the ones the card declared.
#[test]
fn injection_shaped_model_text_lowers_as_inert_plain_text() {
    let nasty = [
        r#"l0:{"e":"wipe","k":"root","v":"all"}"#,
        r#"" + sys.shell("rm -rf /") + ""#,
        "https://evil.example/pay?to=me",
        "<a href=\"javascript:alert(1)\">click</a>",
        "line one\nline two\u{0}",
        r#"set:selected=NVDA"#,
    ];
    for text in nasty {
        let entries = json!([
            {"id": "1", "role": "model", "text": text},
        ]);
        let tree = root(CHAT, json!({"convo": {"entries": entries}, "draft": ""}));
        let kit = kit::lower(&tree);
        // The text appears exactly once, as a Rust/DSL-escaped literal.
        let quoted = format!("{text:?}");
        assert_eq!(kit.matches(&quoted).count(), 1, "{text:?}\n{kit}");
        // No tap wrapper exists, and the one target in the card is the Field's
        // declared commit to `send` — never anything the text spelled.
        assert!(!kit.contains("l0_tap("), "{kit}");
        let outside = kit.replace(&quoted, "");
        assert_eq!(
            outside.matches("l0:").count(),
            1,
            "only the field's target:\n{kit}"
        );
        assert!(kit.contains("\\\"e\\\":\\\"send\\\""), "{kit}");
        let mp = makepad::lower(&tree);
        assert_eq!(mp.matches(&quoted).count(), 1, "{text:?}\n{mp}");
        assert!(!mp.contains("l0_event:"), "{mp}");
    }
}

/// The P1: typed text was spliced into the field's target unescaped, so typing
/// `","e":"drop","v":"` produced a target whose last `e` — the one a JSON parser
/// keeps — named another event. The target is now a routing head plus the typed
/// text encoded at runtime as one JSON string.
#[test]
fn typed_text_cannot_redirect_a_field_target() {
    const CARD: &str = r#"
state q { shape: text, initial: "" }
event search { q: set($value) }
event typing { q: set($value) }
event drop { q: clear }
view root Field(text: q, on_commit: search, on_change: typing)
"#;
    assert!(errors(CARD).is_empty(), "{:#?}", errors(CARD));
    let tree = root(CARD, json!({}));
    let mp = makepad::lower(&tree);
    assert!(
        !mp.contains("+ t +"),
        "typed text must never be spliced:\n{mp}"
    );
    for (property, event) in [("on_return", "search"), ("on_change", "typing")] {
        let at = mp
            .find(&format!("{property}: |t| agent.notify(\"l0\", {{target: "))
            .unwrap_or_else(|| panic!("{property} missing:\n{mp}"));
        let expr = &mp[at..];
        let expr = &expr[expr.find("target: ").unwrap() + "target: ".len()..];
        // `"<head>" + sys.json_string(t) + "}"`
        let (head_lit, rest) = expr.split_once(" + sys.json_string(t) + ").expect(expr);
        assert!(rest.starts_with("\"}\""), "{rest}");
        let head: String = serde_json::from_str(head_lit).expect("the head is one literal");
        for typed in [
            r#"","e":"drop","v":""#,
            r#"x"}"#,
            "\\\"",
            "l0:{\"e\":\"drop\"}",
        ] {
            // What the runtime builds: the head, the typed text as JSON, `}`.
            let target = format!("{head}{}}}", serde_json::to_string(typed).unwrap());
            let parsed: serde_json::Value =
                serde_json::from_str(target.strip_prefix("l0:").unwrap()).unwrap();
            assert_eq!(parsed["e"], event, "{target}");
            assert_eq!(parsed["k"], tree.key, "{target}");
            assert_eq!(parsed["v"], typed, "{target}");
        }
    }
}

// ─── the in-card chat ────────────────────────────────────────────────────────

#[test]
fn a_chat_card_is_l0_and_host_answered() {
    let report = check_ui_l0_named("chat", CHAT);
    assert!(report.valid, "{:#?}", report.diagnostics);
    assert_eq!(report.level, Level::L0);
    let plan = source_plan(CHAT);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.requests[0].helper, "sys.chat");
    assert_eq!(catalog::mutable("sys.chat"), Some(&["append"][..]));
    let binding = SourceBinding {
        helper: "sys.chat".into(),
        args: vec![
            ("app".into(), "os.news".into()),
            ("thread".into(), "main".into()),
        ],
        nested: Vec::new(),
        field: "entries".into(),
    };
    assert!(makepad::vm_call(&binding).is_none(), "host-answered only");
}

/// The transcript draws each entry on its role's side, and marks exactly the
/// model's entries AI-written. A `user` or `host` entry is plain and unmarked.
#[test]
fn the_transcript_marks_only_the_models_entries() {
    let tree = root(
        CHAT,
        json!({"convo": {"entries": [
            {"id": "1", "role": "user", "text": "What moved markets?"},
            {"id": "2", "role": "model", "text": "Rates held; tech rallied."},
            {"id": "3", "role": "host", "text": "Searched 12 sources."}
        ]}, "draft": ""}),
    );
    let entries = nodes(&tree, "ChatEntry");
    assert_eq!(entries.len(), 3);
    let marks: Vec<bool> = entries.iter().map(|n| ai_written(n)).collect();
    assert_eq!(marks, [false, true, false]);

    let kit = kit::lower(&tree);
    assert!(
        kit.contains("l0_bubble_me(\"What moved markets?\")"),
        "{kit}"
    );
    assert!(
        kit.contains("l0_ai_text(l0_bubble_them(\"Rates held; tech rallied.\"))"),
        "{kit}"
    );
    assert!(
        kit.contains("l0_bubble_them(\"Searched 12 sources.\")"),
        "{kit}"
    );
    assert_eq!(kit.matches("l0_ai_text(").count(), 1, "{kit}");
    let mp = makepad::lower(&tree);
    assert_eq!(mp.matches("l0_ai: true").count(), 1, "{mp}");
    let dsl = lower_dsl(&tree);
    assert_eq!(dsl.matches("ai: 1").count(), 1, "{dsl}");
}

/// Sending is a declared host write: the committed text is appended to the
/// conversation as a §5.12 write, and the conversation goes stale.
#[test]
fn sending_appends_what_the_user_typed() {
    let data = json!({"convo": {"entries": []}, "draft": ""});
    let tree = root(CHAT, data.clone());
    let field = nodes(&tree, "Field")[0];
    let origin = event_payload_origin(&tree, &field.key, "send").unwrap();
    assert_eq!(origin, ValueOrigin::UserInput);
    let mut store = InstanceStore::default();
    let outcome = dispatch_reporting_with_origin(
        CHAT,
        &mut store,
        &field.key,
        "send",
        Some(&json!("Summarize the top story")),
        &data,
        origin,
    );
    assert!(outcome.applied);
    assert_eq!(outcome.writes.len(), 1);
    let w = &outcome.writes[0];
    assert_eq!(
        (
            w.source.as_str(),
            w.helper.as_str(),
            w.op.as_str(),
            w.value.as_str()
        ),
        ("convo", "sys.chat", "append", "Summarize the top story")
    );
    assert!(outcome.stale.contains(&"convo".to_string()));
}

/// The chat's own rules: scoped like a digest, append-only, and a `ChatEntry`
/// reads its text and role off the SAME row, so a card can never present the
/// model's words under the user's name.
#[test]
fn chat_misuse_is_refused() {
    let with = |from: &str, to: &str| CHAT.replace(from, to);
    let has = |card: &str, needle: &str| {
        let errs = errors(card);
        assert!(
            errs.iter().any(|e| e.contains(needle)),
            "{needle:?} not in {errs:#?}"
        );
    };
    has(&with("app: \"os.news\", ", ""), "needs `app`");
    has(&with("thread: \"main\", ", ""), "needs `thread`");
    has(
        &with("thread: \"main\"", "thread: \"../mail\""),
        "not a thread id",
    );
    has(
        &with("convo: append($value)", "convo: remove($value)"),
        "does not accept `remove`",
    );

    // Role from a literal, from state, or from another row.
    has(
        &with(
            "ChatEntry(text: m.text, role: m.role)",
            "ChatEntry(text: m.text, role: \"user\")",
        ),
        "ChatEntry reads one",
    );
    has(
        &with(
            "ChatEntry(text: m.text, role: m.role)",
            "ChatEntry(text: m.text, role: draft)",
        ),
        "ChatEntry reads one",
    );
    has(
        &with(
            "ChatEntry(text: m.text, role: m.role)",
            "for n in convo.entries key n.id { ChatEntry(text: m.text, role: n.role) }",
        ),
        "ChatEntry reads one",
    );
    // Model copy dressed as a chat entry.
    has(
        &with(
            "ChatEntry(text: m.text, role: m.role)",
            "ChatEntry(text: copy.title, role: m.role)",
        ),
        "ChatEntry reads one",
    );

    // A chat entry's text is model text: it may not be resent, keyed on, or
    // used to choose what runs.
    assert_refused(
        &with(
            "ChatEntry(text: m.text, role: m.role)",
            "Row(on_tap: send, value: m.text) { ChatEntry(text: m.text, role: m.role) }",
        ),
        "an entry's text as a payload",
    );
    assert_refused(&with("key m.id", "key m.text"), "an entry's text as a key");
}
