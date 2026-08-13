//! Contract test: every Tauri command the frontend invokes must be registered
//! in `lib.rs`'s `generate_handler![...]`.
//!
//! This exists because a missing registration produces **no compile error on
//! either side** — `invoke()` takes a plain string, so TypeScript is happy, and
//! Rust never learns the frontend wanted the command. The only symptom is a
//! runtime "command not found", which UI code frequently swallows into an empty
//! render. Phase 10 shipped six such mismatches before this test existed.
//!
//! Integration tests run with the crate root (`src-tauri/`) as CWD.

use std::collections::BTreeSet;
use std::fs;

const TAURI_TS: &str = "../src/lib/tauri.ts";
const LIB_RS: &str = "src/lib.rs";

/// Extracts command names from every `invoke<T>("command_name", ...)` call.
fn invoked_command_names(source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let bytes = source.as_bytes();
    let mut search_from = 0;

    while let Some(rel) = source[search_from..].find("invoke<") {
        let idx = search_from + rel;
        search_from = idx + "invoke<".len();

        // Walk forward to the call's opening paren, then to the first quote.
        let Some(paren_rel) = source[search_from..].find('(') else {
            break;
        };
        let after_paren = search_from + paren_rel + 1;

        // The first non-whitespace character must be the opening quote of a
        // string literal; anything else means this is a dynamic call we cannot
        // statically verify, so skip it.
        let mut cursor = after_paren;
        while cursor < bytes.len() && (bytes[cursor] as char).is_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'"' {
            continue;
        }
        let start = cursor + 1;
        let Some(end_rel) = source[start..].find('"') else {
            break;
        };
        names.insert(source[start..start + end_rel].to_string());
    }

    names
}

/// Extracts the last path segment of every entry in `generate_handler![...]`.
fn registered_command_names(source: &str) -> BTreeSet<String> {
    let Some(start) = source.find("generate_handler![") else {
        panic!("could not find generate_handler![ in {LIB_RS}");
    };
    let body_start = start + "generate_handler![".len();

    // Depth-count to the matching bracket so nested brackets cannot truncate us.
    let mut depth = 1usize;
    let mut end = body_start;
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + offset;
                    break;
                }
            }
            _ => {}
        }
    }

    source[body_start..end]
        .split(',')
        .filter_map(|entry| {
            let cleaned: String = entry
                .lines()
                // Drop line comments so commented-out entries never count as registered.
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("");
            let name = cleaned.rsplit("::").next()?.trim().to_string();
            if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

#[test]
fn every_invoked_command_is_registered() {
    let ts = fs::read_to_string(TAURI_TS)
        .unwrap_or_else(|e| panic!("failed to read {TAURI_TS}: {e}"));
    let rs = fs::read_to_string(LIB_RS)
        .unwrap_or_else(|e| panic!("failed to read {LIB_RS}: {e}"));

    let invoked = invoked_command_names(&ts);
    let registered = registered_command_names(&rs);

    assert!(
        !invoked.is_empty(),
        "parsed zero invoke() calls from {TAURI_TS} — the parser is broken, not the code"
    );
    assert!(
        !registered.is_empty(),
        "parsed zero registered commands from {LIB_RS} — the parser is broken, not the code"
    );

    let missing: Vec<&String> = invoked.difference(&registered).collect();

    assert!(
        missing.is_empty(),
        "\n{} frontend command(s) are invoked but NOT registered in {}'s \
         generate_handler![...].\nEach will fail at runtime with \"command not found\":\n{}\n\n\
         Fix: add the command to generate_handler![...], or correct the name in {}.\n",
        missing.len(),
        LIB_RS,
        missing
            .iter()
            .map(|n| format!("  - {n}"))
            .collect::<Vec<_>>()
            .join("\n"),
        TAURI_TS,
    );
}

#[test]
fn parsers_extract_known_good_samples() {
    // Guards the parsers themselves: if these regress, the contract test above
    // could pass vacuously.
    let ts_sample = r#"
        export const getTasks = () => invoke<Task[]>("get_tasks_for_project", { projectId });
        export const noArgs = () => invoke<void>("do_thing");
        export const spaced = () => invoke<Foo>(  "spaced_name"  , {});
    "#;
    let names = invoked_command_names(ts_sample);
    assert!(names.contains("get_tasks_for_project"));
    assert!(names.contains("do_thing"));
    assert!(names.contains("spaced_name"));
    assert_eq!(names.len(), 3);

    let rs_sample = r#"
        .invoke_handler(tauri::generate_handler![
            commands::tasks::get_tasks_for_project,
            commands::ai::chat_with_project,
            // commands::dead::not_registered,
            greet
        ])
    "#;
    let registered = registered_command_names(rs_sample);
    assert!(registered.contains("get_tasks_for_project"));
    assert!(registered.contains("chat_with_project"));
    assert!(registered.contains("greet"));
    assert!(
        !registered.contains("not_registered"),
        "commented-out entries must not count as registered"
    );
}
