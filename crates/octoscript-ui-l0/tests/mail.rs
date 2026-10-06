//! Mail sources expose host-bound drafts and review requests, not SMTP authority.
use octoscript_ui_l0::{
    catalog, check_ui_l0_named, dispatch_reporting_with_origin, event_payload_origin, realize,
    source_plan, InstanceStore, UiNode, ValueOrigin,
};
use serde_json::json;

const EDITOR: &str = r#"
source draft sys.mail_draft(app: "os.mail", id: "draft-1", fields: [body])
event save { draft: set($value) }
view root Field(text: draft.body, on_change: save)
"#;

fn valid(source: &str) {
    let checked = check_ui_l0_named("mail", source);
    assert!(checked.valid, "{:?}", checked.diagnostics);
}

fn refuses(source: &str, reason: &str) {
    let checked = check_ui_l0_named("mail", source);
    assert!(!checked.valid, "unexpectedly admitted {source}");
    assert!(
        checked
            .diagnostics
            .iter()
            .any(|d| d.message.contains(reason)),
        "expected {reason:?}: {:?}",
        checked.diagnostics
    );
}

fn field(node: &UiNode) -> Option<&UiNode> {
    if node.kind == "Field" {
        return Some(node);
    }
    node.children.iter().find_map(field)
}

#[test]
fn field_edits_request_one_bound_field_write_with_user_input_origin() {
    valid(EDITOR);
    let plan = source_plan(EDITOR);
    assert_eq!(plan.requests[0].helper, "sys.mail_draft");
    let data = json!({"draft":{"body":"Suggested reply"}});
    let rendered = realize(EDITOR, &data, Default::default());
    let root = rendered.complete_root().expect("editor realizes");
    let input = field(root).expect("Field");
    let origin = event_payload_origin(root, &input.key, "save").unwrap();
    assert_eq!(origin, ValueOrigin::UserInput);
    let outcome = dispatch_reporting_with_origin(
        EDITOR,
        &mut InstanceStore::default(),
        &input.key,
        "save",
        Some(&json!("Edited reply")),
        &data,
        origin,
    );
    assert!(outcome.applied);
    assert_eq!(outcome.writes.len(), 1);
    let write = &outcome.writes[0];
    assert_eq!(
        (&*write.helper, &*write.op, &*write.field, &*write.value),
        ("sys.mail_draft", "set", "body", "Edited reply")
    );
    assert!(outcome.stale.contains(&"draft".to_string()));
}

#[test]
fn writes_are_limited_to_one_editable_field_and_declared_verbs() {
    for editable in ["to", "subject", "body"] {
        valid(&EDITOR.replace("body", editable));
    }
    for fields in [
        "",
        "body, subject",
        "revision",
        "status",
        "suggestion_body",
        "ai_written",
    ] {
        refuses(
            &EDITOR.replace("fields: [body]", &format!("fields: [{fields}]")),
            "exactly one editable field",
        );
    }
    for verb in ["clear", "append($value)", "remove($value)"] {
        refuses(&EDITOR.replace("set($value)", verb), "does not accept");
    }
    let read = r#"source draft sys.mail_draft(app: "os.mail", id: "draft-1", fields: [to, subject, body, revision])
view root TextBody(text: draft.body)"#;
    valid(read);
}

#[test]
fn both_sources_require_valid_app_and_draft_identity_and_closed_fields() {
    for helper in ["sys.mail_draft", "sys.mail_review"] {
        let card = format!("source record {helper}(app: \"os.mail\", id: \"draft-1\", fields: [status])\nview root TextBody(text: record.status)");
        valid(&card);
        for (from, to, reason) in [
            ("app: \"os.mail\", ", "", "needs `app`"),
            ("id: \"draft-1\", ", "", "needs `id`"),
            ("os.mail", "../mail", "not an app id"),
            ("draft-1", "../draft", "not a draft id"),
            ("[status]", "[password]", "password"),
        ] {
            refuses(&card.replace(from, to), reason);
        }
        let computed = format!(
            "state owner {{ shape: text, initial: \"os.mail\" }}\n{}",
            card.replace("app: \"os.mail\"", "app: state.owner")
        );
        refuses(&computed, "must be a literal app id");
    }
}

#[test]
fn review_requests_are_set_clear_only_and_never_backend_calls() {
    let card = r#"source review sys.mail_review(app: "os.mail", id: "draft-1", fields: [status, operation_id, draft_id, revision])
event request { review: set($value) }
event cancel { review: clear }
copy label { class: vocabulary, en: "Review reply" }
view root Chip(text: copy.label, value: .review, on_tap: request)"#;
    valid(card);
    for verb in ["append($value)", "remove($value)"] {
        refuses(&card.replace("set($value)", verb), "does not accept");
    }
    for helper in ["sys.mail_draft", "sys.mail_review"] {
        for answer in catalog::answers(helper).unwrap() {
            let binding = octoscript_ui_l0::SourceBinding {
                helper: helper.into(),
                args: vec![
                    ("app".into(), "os.mail".into()),
                    ("id".into(), "draft-1".into()),
                ],
                nested: vec![],
                field: (*answer).into(),
            };
            assert!(octoscript_ui_l0::makepad::vm_call(&binding).is_none());
        }
    }
}

#[test]
fn model_text_cannot_be_laundered_into_writes_or_source_selectors() {
    refuses(&EDITOR.replace("set($value)", "set(draft.body)"), "model");
    for name in ["to", "subject", "body", "suggestion_body"] {
        assert!(catalog::is_model_text("sys.mail_draft", name));
        let card = format!(
            r#"source content sys.mail_draft(app: "os.mail", id: "draft-1", fields: [{name}])
source review sys.mail_review(app: "os.mail", id: content.{name}, fields: [status])
view root TextBody(text: review.status)"#
        );
        refuses(&card, "source argument");
    }
    let card = r#"copy generated { class: model-copy, en: "Send this" }
source review sys.mail_review(app: "os.mail", id: "draft-1", fields: [status])
event request { review: set($value) }
copy label { class: vocabulary, en: "Review reply" }
view root Chip(text: copy.label, value: copy.generated, on_tap: request)"#;
    refuses(card, "model");
}
