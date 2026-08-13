//! Shape contract: pins the exact JSON that boundary types serialize to.
//!
//! Rust enums crossing the Tauri boundary are the sharpest edge here. Serde's
//! default is *external* tagging, which emits `{"Learning":{...}}` for struct
//! variants and a bare `"Ready"` string for unit variants. TypeScript consumers
//! in this codebase model these as discriminated unions (`{"type":"Learning"}`),
//! and nothing on either side catches the mismatch: Rust does not know the TS
//! type, and TS trusts whatever `invoke<T>()` claims.
//!
//! Phase 10 shipped three such mismatches. These tests fail loudly if the wire
//! format drifts from what `src/lib/tauri.ts` declares.

use meridian_lib::productivity::models::{
    Confidence, ProductivityStatus, TimeSuggestion,
};
use meridian_lib::role::models::InferenceStatus;
use serde_json::{json, Value};

fn to_value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).expect("serialization failed")
}

#[test]
fn inference_status_is_internally_tagged() {
    assert_eq!(
        to_value(&InferenceStatus::Confirmed {
            role: "tech_lead".into(),
            secondary: Some("ic".into()),
        }),
        json!({ "type": "Confirmed", "role": "tech_lead", "secondary": "ic" }),
        "InferenceStatus must carry a `type` discriminant — see tauri.ts InferenceStatus"
    );

    assert_eq!(
        to_value(&InferenceStatus::Learning {
            message: "Getting to know your role...".into(),
            progress: 40.0,
        }),
        json!({ "type": "Learning", "message": "Getting to know your role...", "progress": 40.0 }),
    );

    // 0.5 is exactly representable as f32; an inexact literal like 0.62 widens
    // to 0.6200000047683716 on the way to f64 and would fail on precision
    // rather than on shape, which is what this test is actually pinning.
    assert_eq!(
        to_value(&InferenceStatus::PendingConfirmation {
            inferred: "ic".into(),
            confidence: 0.5,
        }),
        json!({ "type": "PendingConfirmation", "inferred": "ic", "confidence": 0.5 }),
    );
}

#[test]
fn productivity_status_unit_variants_are_objects_not_strings() {
    // The regression that matters: without `#[serde(tag = "type")]` these
    // serialize as the bare strings "Ready" / "Disabled", and every
    // `status.type === "Ready"` check in the UI silently evaluates false.
    assert_eq!(
        to_value(&ProductivityStatus::Ready),
        json!({ "type": "Ready" }),
    );
    assert_eq!(
        to_value(&ProductivityStatus::Disabled),
        json!({ "type": "Disabled" }),
    );
    assert_eq!(
        to_value(&ProductivityStatus::Learning {
            completions_needed: 23
        }),
        json!({ "type": "Learning", "completions_needed": 23 }),
    );
}

#[test]
fn time_suggestion_shape_matches_typescript() {
    let value = to_value(&TimeSuggestion {
        suggested_hour: 9,
        reason: "You typically complete focus work best in the morning".into(),
        confidence: Confidence::High,
    });

    assert_eq!(
        value,
        json!({
            "suggested_hour": 9,
            "reason": "You typically complete focus work best in the morning",
            "confidence": "High"
        }),
    );

    // Guards the specific bug: TS previously declared `peak_hours` / `avoid_hours`
    // and treated `confidence` as a number. Reading `.length` off the absent
    // arrays threw at runtime.
    let obj = value.as_object().unwrap();
    assert!(
        !obj.contains_key("peak_hours") && !obj.contains_key("avoid_hours"),
        "TimeSuggestion gained fields — update the TS interface in tauri.ts to match"
    );
    assert!(
        obj["confidence"].is_string(),
        "confidence is the Confidence enum, not a numeric score"
    );
}

#[test]
fn confidence_variants_serialize_as_plain_strings() {
    assert_eq!(to_value(&Confidence::High), json!("High"));
    assert_eq!(to_value(&Confidence::Default), json!("Default"));
    assert_eq!(to_value(&Confidence::Low), json!("Low"));
}

#[test]
fn message_type_vocabulary_is_pinned() {
    use meridian_lib::messages::models::MessageType;

    // These four strings are duplicated in three frontend places:
    //   - the `Message.message_type` union in src/lib/tauri.ts
    //   - TYPE_LABELS / TYPE_ICONS in src/components/messages/MessageCard.tsx
    //   - MESSAGE_TYPES in src/components/messages/MessageFilters.tsx
    // Adding a variant here without updating those renders the raw snake_case
    // identifier as the badge and leaves the type unfilterable.
    let vocabulary: Vec<&str> = vec![
        MessageType::SkillResult.as_str(),
        MessageType::Digest.as_str(),
        MessageType::PinnedChat.as_str(),
        MessageType::IntegrationSync.as_str(),
    ];

    assert_eq!(
        vocabulary,
        vec!["skill_result", "digest", "pinned_chat", "integration_sync"],
        "MessageType vocabulary changed — update tauri.ts, MessageCard.tsx and MessageFilters.tsx"
    );

    // Round-trips, so a string written by one producer parses back.
    for name in &vocabulary {
        assert!(
            MessageType::from_str(name).is_some(),
            "{name} does not round-trip through MessageType::from_str"
        );
    }
}
