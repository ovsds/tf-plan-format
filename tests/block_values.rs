use tf_plan_format::template::tera::{render_diff_values, render_values};
use tf_plan_format::tf::{Value, ValueMap};

fn values(value: Value) -> ValueMap {
    ValueMap::from([("value".to_string(), value)])
}

fn string(value: &str) -> Value {
    Value::String(value.to_string())
}

fn diff(before: &str, after: &str, show: bool) -> String {
    render_diff_values(
        Some(&values(string(before))),
        Some(&values(string(after))),
        show,
    )
}

#[test]
fn structured_strings_create_and_delete_as_blocks() {
    for document in [
        r#"{"replicas":2,"service":{"ports":[80,443]}}"#,
        "replicas: 2\nservice:\n  ports: [80, 443]\n",
    ] {
        let map = values(string(document));
        let lines = [
            "value:",
            "  replicas: 2",
            "  service:",
            "    ports:",
            "      - 80",
            "      - 443",
        ];
        for show in [false, true] {
            for (before, after, marker) in [(None, Some(&map), '+'), (Some(&map), None, '-')] {
                let expected = lines
                    .iter()
                    .map(|line| format!("{marker} {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert_eq!(
                    expected,
                    render_diff_values(before, after, show),
                    "{document}"
                );
                assert_eq!(lines.join("\n"), render_values(before, after, show));
            }
        }
    }
}

#[test]
fn document_updates_compare_structure() {
    for (before, after) in [
        (
            r#"{"replicas":1,"enabled":true}"#,
            "enabled: true\nreplicas: 2\n",
        ),
        (
            "replicas: 1\nenabled: true\n",
            r#"{"enabled":true,"replicas":2}"#,
        ),
    ] {
        assert_eq!(
            "  value:\n    enabled: true\n!   replicas: 1 -> 2",
            diff(before, after, true)
        );
        assert_eq!("  value:\n!   replicas: 1 -> 2", diff(before, after, false));
    }
}

#[test]
fn document_formatting_and_key_order_are_not_changes() {
    let before = r#"{"a":1,"nested":{"b":2,"a":1}}"#;
    let after = "nested:\n  a: 1\n  b: 2\na: 1\n";
    assert_eq!("", diff(before, after, false));
    let shown = diff(before, after, true);
    assert!(!shown.contains('!'));
    assert!(!shown.contains('+'));
    assert!(!shown.contains('-'));
    assert!(shown.contains("nested:\n      a: 1\n      b: 2"));
}

#[test]
fn multiline_text_changes_use_line_additions_and_removals() {
    assert_eq!(
        "  value: |\n    first\n-   old\n+   new\n    last",
        diff("first\nold\nlast", "first\nnew\nlast", true)
    );
    let hidden = diff("first\nold\nlast", "first\nnew\nlast", false);
    assert!(hidden.contains("-   old\n+   new"), "{hidden}");
    assert!(!hidden.contains("first"));
    assert!(!hidden.contains("last"));
}

#[test]
fn literal_text_preserves_blank_and_boundary_lines() {
    let map = values(string("\nfirst\n\nlast\n"));
    assert_eq!(
        "value: |\n  \n  first\n  \n  last\n  ",
        render_values(None, Some(&map), true)
    );
    let trailing = diff("first\nlast", "first\nlast\n", true);
    assert!(trailing.contains("+   "), "{trailing}");
    let leading = diff("first\nlast", "\nfirst\nlast", true);
    assert!(leading.contains("+   "), "{leading}");
}

#[test]
fn document_detection_rejects_lossy_or_ambiguous_input() {
    for document in [
        "{\"a\":1,\"a\":2}",
        "a: 1\na: 2\n",
        "1: value\n",
        "? [a, b]\n: value\n",
        "a: !custom value\n",
        "a: 1\n---\nb: 2\n",
        "a: [broken\n",
        "a: .nan\n",
        "a: .inf\n",
        "a: 18446744073709551616\n",
        "{\"a\":18446744073709551616}",
    ] {
        let rendered = render_values(None, Some(&values(string(document))), true);
        if document.contains('\n') {
            assert!(
                rendered.starts_with("value: |\n"),
                "{document:?}: {rendered}"
            );
            assert_eq!(
                document,
                rendered
                    .strip_prefix("value: |\n")
                    .unwrap()
                    .split('\n')
                    .map(|line| line.strip_prefix("  ").unwrap())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        } else {
            assert_eq!(
                format!("value: {}", serde_json::to_string(document).unwrap()),
                rendered
            );
        }
    }
}

#[test]
fn scalar_roots_stay_strings() {
    for document in ["null", "true", "42", "3.14", "plain", "\"quoted\""] {
        assert_eq!(
            format!("value: {}", serde_json::to_string(document).unwrap()),
            render_values(None, Some(&values(string(document))), true)
        );
    }
}

#[test]
fn document_string_leaves_are_not_recursively_interpreted() {
    let map = values(string(r#"{"embedded":"{\"a\":1}","yaml":"a: 1\nb: 2"}"#));
    let rendered = render_values(None, Some(&map), true);
    assert!(
        rendered.contains("embedded: \"{\\\"a\\\":1}\""),
        "{rendered}"
    );
    assert!(
        rendered.contains("yaml: |\n    a: 1\n    b: 2"),
        "{rendered}"
    );
}

#[test]
fn arrays_expand_recursively_in_whole_value_changes() {
    let map = values(Value::Array(vec![
        Value::Array(vec![Value::Integer(1), Value::Array(vec![])]),
        Value::Object(ValueMap::new()),
        Value::Array(vec![]),
    ]));
    let expected = "value:\n  - - 1\n    - []\n  - {}\n  - []";
    assert_eq!(expected, render_values(None, Some(&map), true));
    assert_eq!(expected, render_values(Some(&map), None, true));
    let empty = ValueMap::new();
    for show in [false, true] {
        let created = render_diff_values(Some(&empty), Some(&map), show);
        assert_eq!(
            expected
                .lines()
                .map(|line| format!("! {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            created
        );
        let deleted = render_diff_values(Some(&map), Some(&empty), show);
        assert_eq!(
            expected
                .lines()
                .map(|line| format!("! {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            deleted
        );
    }
}

#[test]
fn documents_preserve_absent_and_explicit_null_members_and_elements() {
    for show in [false, true] {
        assert_eq!(
            "  value:\n+   item: null",
            diff("{}", r#"{"item":null}"#, show)
        );
        assert_eq!(
            "  value:\n-   item: null",
            diff(r#"{"item":null}"#, "{}", show)
        );
        assert_eq!("  value:\n+   - null", diff("[]", "[null]", show));
        assert_eq!("  value:\n-   - null", diff("[null]", "[]", show));
    }
}

#[test]
fn opaque_transitions_never_expand_previous_document() {
    for opaque in [
        Value::Sensitive,
        Value::SensitiveChanged,
        Value::SensitiveUnknown,
        Value::Unknown,
    ] {
        for show in [false, true] {
            let document = values(string("secret: never-disclose\n"));
            let marker = values(opaque.clone());
            for (before, after) in [(&document, &marker), (&marker, &document)] {
                let result = render_diff_values(Some(before), Some(after), show);
                assert!(!result.contains('\n'), "{result}");
                assert!(result.starts_with("! value: "), "{result}");
            }
        }
    }
}

#[test]
fn plain_and_diff_renderers_share_identical_value_lines() {
    let before = values(Value::Array(vec![
        string("replicas: 1\n"),
        string("first\nold"),
    ]));
    let after = values(Value::Array(vec![
        string("replicas: 2\n"),
        string("first\nnew"),
        Value::Null,
    ]));
    for show in [false, true] {
        for (before, after) in [
            (Some(&before), Some(&after)),
            (None, Some(&after)),
            (Some(&before), None),
        ] {
            let diff = render_diff_values(before, after, show);
            let plain = diff
                .lines()
                .map(|line| &line[2..])
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(plain, render_values(before, after, show));
        }
    }
}

#[test]
fn document_and_text_type_changes_keep_both_original_strings() {
    for (before, after) in [("a: 1\n", "plain\ntext"), ("plain\ntext", "a: 1\n")] {
        let result = diff(before, after, true);
        assert!(result.contains("-   "), "{result}");
        assert!(result.contains("+   "), "{result}");
        assert!(result.contains("a: 1"), "{result}");
        assert!(result.contains("plain"), "{result}");
    }
}

#[test]
fn native_nested_arrays_preserve_opaque_markers() {
    let map = values(Value::Array(vec![Value::Array(vec![
        Value::Sensitive,
        Value::Unknown,
        Value::SensitiveUnknown,
    ])]));
    let result = render_diff_values(None, Some(&map), true);
    assert_eq!("+ value:\n+   - - sensitive\n+     - (known after apply)\n+     - sensitive (known after apply)", result);
}

#[test]
fn formatting_only_document_changes_inside_native_arrays_are_unchanged() {
    let before = values(Value::Array(vec![string(r#"{"a":1,"b":{"c":2}}"#)]));
    let after = values(Value::Array(vec![string("b:\n  c: 2\na: 1\n")]));
    assert_eq!("", render_diff_values(Some(&before), Some(&after), false));
    assert_eq!("", render_values(Some(&before), Some(&after), false));
    let shown = render_diff_values(Some(&before), Some(&after), true);
    assert_ne!(shown, "");
    assert!(shown.lines().all(|line| line.starts_with("  ")), "{shown}");
    assert!(shown.contains("a: 1"), "{shown}");
    assert!(shown.contains("c: 2"), "{shown}");
}

#[test]
fn document_keys_escape_empty_punctuation_and_physical_line_breaks() {
    let before = r#"{"line\nbreak":1,"":1,"a: b":1,"simple-key":1}"#;
    let after = r#"{"line\nbreak":2,"":2,"a: b":2,"simple-key":2}"#;
    let created = values(string(before));
    assert_eq!(
        "+ value:\n+   \"\": 1\n+   \"a: b\": 1\n+   \"line\\nbreak\": 1\n+   simple-key: 1",
        render_diff_values(None, Some(&created), true)
    );
    for show in [false, true] {
        assert_eq!(
            "  value:\n!   \"\": 1 -> 2\n!   \"a: b\": 1 -> 2\n!   \"line\\nbreak\": 1 -> 2\n!   simple-key: 1 -> 2",
            diff(before, after, show)
        );
    }
}

#[test]
fn native_array_alignment_preserves_documents_with_formatting_changes() {
    let before = values(Value::Array(vec![string(r#"{"a":1}"#), string("end")]));
    let after = values(Value::Array(vec![
        string(r#"{"new":2}"#),
        string("a: 1\n"),
        string("end"),
    ]));
    for (before, after, direction) in [(&before, &after, true), (&after, &before, false)] {
        for show in [false, true] {
            let expected = if show {
                "  value:\n!   - new: 2\n    - a: 1\n    - \"end\""
            } else {
                "  value:\n!   - new: 2\n    - ..."
            };
            assert_eq!(
                expected,
                render_diff_values(Some(before), Some(after), show),
                "insertion={direction}, show={show}"
            );
        }
    }
}

#[test]
fn json_unicode_surrogate_pairs_parse_without_yaml_escape_rules() {
    let document = r#"{"emoji":"\ud83d\ude00","nested":[{"count":1.25}]}"#;
    assert_eq!(
        "value:\n  emoji: \"😀\"\n  nested:\n    - count: 1.25",
        render_values(None, Some(&values(string(document))), true)
    );
    assert_eq!(
        "",
        diff(document, "emoji: 😀\nnested:\n  - count: 1.25\n", false)
    );
    for document in [
        r#"{"emoji":"\ud83d\ude00","n":0.10000000000000000001}"#,
        r#"{"emoji":"\ud83d\ude00","n":18446744073709551616}"#,
        r#"{"emoji":"\ud83d\ude00","n":1,"n":2}"#,
        r#"{"nested":{"n":0.10000000000000000001}}"#,
    ] {
        assert_eq!(
            format!("value: {}", serde_json::to_string(document).unwrap()),
            render_values(None, Some(&values(string(document))), true)
        );
    }
}

#[test]
fn unchanged_native_arrays_expand_document_and_text_strings() {
    let array = values(Value::Array(vec![
        string(r#"{"a":1}"#),
        string("first\nlast"),
        Value::Object(ValueMap::from([(
            "nested".to_string(),
            Value::Array(vec![string("replicas: 2")]),
        )])),
    ]));
    assert_eq!(
        "  value:\n    - a: 1\n    - |\n      first\n      last\n    - nested:\n        - replicas: 2",
        render_diff_values(Some(&array), Some(&array), true)
    );
    assert_eq!("", render_diff_values(Some(&array), Some(&array), false));
}
