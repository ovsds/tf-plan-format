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
    for (document, lines) in [
        (
            r#"{"replicas":2,"service":{"ports":[80,443]}}"#,
            &[
                "value: | json",
                "  {",
                "    \"replicas\": 2,",
                "    \"service\": {",
                "      \"ports\": [",
                "        80,",
                "        443",
                "      ]",
                "    }",
                "  }",
            ][..],
        ),
        (
            "replicas: 2\nservice:\n  ports: [80, 443]\n",
            &[
                "value: | yaml",
                "  replicas: 2",
                "  service:",
                "    ports: [80, 443]",
            ][..],
        ),
    ] {
        let map = values(string(document));
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
fn document_updates_diff_by_line() {
    let before = r#"{"replicas":1,"enabled":true}"#;
    let after = r#"{"replicas":2,"enabled":true}"#;
    assert_eq!(
        "  value: | json\n    {\n-     \"replicas\": 1,\n+     \"replicas\": 2,\n      \"enabled\": true\n    }",
        diff(before, after, true)
    );
    assert_eq!(
        "  value: | json\n    ...\n-     \"replicas\": 1,\n+     \"replicas\": 2,\n    ...",
        diff(before, after, false)
    );
    assert_eq!(
        "  value: | yaml\n-   replicas: 1\n+   replicas: 2\n    enabled: true",
        diff(
            "replicas: 1\nenabled: true\n",
            "replicas: 2\nenabled: true\n",
            true
        )
    );
}

#[test]
fn json_whitespace_is_no_change_but_key_order_is() {
    let before = r#"{"a":1,"b":2}"#;
    assert_eq!("", diff(before, "{ \"a\": 1,\n  \"b\": 2 }", false));
    assert_eq!(
        "  value: | json\n    ...\n-     \"a\": 1,\n-     \"b\": 2\n+     \"b\": 2,\n+     \"a\": 1\n    ...",
        diff(before, r#"{"b":2,"a":1}"#, false)
    );
}

#[test]
fn json_renders_its_tokens_as_written() {
    let document = r#"{"a":[],"b":{},"c":"x,{y}:\"z\"","d":1.50e3,"e":[{}],"f":"😀"}"#;
    assert_eq!(
        r#"value: | json
  {
    "a": [],
    "b": {},
    "c": "x,{y}:\"z\"",
    "d": 1.50e3,
    "e": [
      {}
    ],
    "f": "😀"
  }"#,
        render_values(None, Some(&values(string(document))), true)
    );
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
    assert_eq!(
        "value: | json\n  {\n    \"embedded\": \"{\\\"a\\\":1}\",\n    \"yaml\": \"a: 1\\nb: 2\"\n  }",
        render_values(None, Some(&map), true)
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
fn opaque_transitions_show_the_document_as_removed_or_added() {
    for (opaque, inline) in [
        (Value::Sensitive, "sensitive"),
        (Value::SensitiveChanged, "sensitive (changed)"),
        (Value::SensitiveUnknown, "sensitive (known after apply)"),
        (Value::Unknown, "(known after apply)"),
    ] {
        let document = values(string("replicas: 2\n"));
        let marker = values(opaque);
        for show in [false, true] {
            assert_eq!(
                format!("- value: | yaml\n-   replicas: 2\n+ value: {inline}"),
                render_diff_values(Some(&document), Some(&marker), show)
            );
            assert_eq!(
                format!("- value: {inline}\n+ value: | yaml\n+   replicas: 2"),
                render_diff_values(Some(&marker), Some(&document), show)
            );
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
fn format_changes_show_both_strings_as_written() {
    for show in [false, true] {
        assert_eq!(
            "- value: | yaml\n-   a: 1\n+ value: |\n+   plain\n+   text",
            diff("a: 1\n", "plain\ntext", show)
        );
        assert_eq!(
            "- value: | json\n-   {\n-     \"a\": 1\n-   }\n+ value: | yaml\n+   a: 1",
            diff(r#"{"a":1}"#, "a: 1\n", show)
        );
        assert_eq!(
            "- value: \"plain\"\n+ value: | json\n+   [\n+     1\n+   ]",
            diff("plain", "[1]", show)
        );
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
fn whitespace_only_json_changes_inside_native_arrays_are_unchanged() {
    let before = values(Value::Array(vec![string(r#"{"a":1,"b":{"c":2}}"#)]));
    let after = values(Value::Array(vec![string(
        "{ \"a\": 1, \"b\": { \"c\": 2 } }",
    )]));
    assert_eq!("", render_diff_values(Some(&before), Some(&after), false));
    assert_eq!("", render_values(Some(&before), Some(&after), false));
    let shown = render_diff_values(Some(&before), Some(&after), true);
    assert_ne!(shown, "");
    assert!(shown.lines().all(|line| line.starts_with("  ")), "{shown}");
    assert!(shown.contains("\"a\": 1"), "{shown}");
    assert!(shown.contains("\"c\": 2"), "{shown}");
}

#[test]
fn native_array_alignment_matches_documents_with_whitespace_changes() {
    let before = values(Value::Array(vec![string(r#"{"a":1}"#), string("end")]));
    let after = values(Value::Array(vec![
        string(r#"{"new":2}"#),
        string(r#"{ "a": 1 }"#),
        string("end"),
    ]));
    for (before, after, direction) in [(&before, &after, true), (&after, &before, false)] {
        for show in [false, true] {
            let expected = if show {
                "  value:\n!   - | json\n!     {\n!       \"new\": 2\n!     }\n    - | json\n      {\n        \"a\": 1\n      }\n    - \"end\""
            } else {
                "  value:\n!   - | json\n!     {\n!       \"new\": 2\n!     }\n    - ..."
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
        "  value:\n    - | json\n      {\n        \"a\": 1\n      }\n    - |\n      first\n      last\n    - nested:\n        - | yaml\n          replicas: 2",
        render_diff_values(Some(&array), Some(&array), true)
    );
    assert_eq!("", render_diff_values(Some(&array), Some(&array), false));
}
