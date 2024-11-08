use itertools::Itertools;

use crate::tf;
use crate::types;
use core::str;
use std::collections::HashSet;

const INDENT_STR: &str = "  ";

pub const GITHUB_MARKDOWN_TEMPLATE: &str = "
{%- for plan_key, plan in data.plans %}<details>
<summary>{{ render_actions(actions=plan.unique_actions) }}{{ plan_key }}</summary>
{%- if not plan.changes %}
No resource changes
{%- else %}
{%- for change in plan.changes %}
<details>
<summary>{{ render_action(action=change.action) }}{{ change.raw.address }}
</summary>

```
{{ render_changes(change=change.raw.change) }}
```

</details>
{%- endfor %}
{%- endif %}
</details>
{% endfor %}";

type Args = std::collections::HashMap<String, tera::Value>;

fn _render_action(action: &tf::Action) -> String {
    match action {
        tf::Action::Create => "✅".to_string(),
        tf::Action::Delete => "❌".to_string(),
        tf::Action::DeleteCreate => "♻️".to_string(),
        tf::Action::Update => "🔄".to_string(),
        tf::Action::NoOp => "🟰".to_string(),
        tf::Action::Read => "🔍".to_string(),
        tf::Action::Unknown => "❓".to_string(),
    }
}

fn render_action(args: &Args) -> tera::Result<tera::Value> {
    let action = args.get("action").ok_or("action must be present in args")?;
    let action = tera::from_value::<tf::Action>(action.clone())?;

    Ok(tera::Value::String(_render_action(&action)))
}

fn render_actions(args: &Args) -> tera::Result<tera::Value> {
    let actions = args
        .get("actions")
        .ok_or("actions must be present in args")?;
    let actions = tera::from_value::<Vec<tf::Action>>(actions.clone())?;

    let result: Vec<String> = actions.iter().map(_render_action).collect();

    Ok(tera::Value::String(result.join("")))
}

fn _render_plaintext(value: &tf::RawValue) -> String {
    format!("{}", tera::to_value(value).unwrap())
}

fn _render_unchanged_plaintext(
    key: &str,
    value: &tf::RawValue,
    indent_count: usize,
) -> Vec<String> {
    vec![format!(
        "{}{key}: {}",
        INDENT_STR.repeat(indent_count),
        _render_plaintext(value)
    )]
}

fn _render_unchanged_hashmap_value(
    value: &std::collections::HashMap<String, tf::RawValue>,
    indent_count: usize,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for (key, value) in value.iter().sorted_by_key(|x| x.0) {
        result.extend(_render_unchanged(key, value, indent_count));
    }
    result
}

fn _render_unchanged_hashmap(
    key: &str,
    value: &std::collections::HashMap<String, tf::RawValue>,
    indent_count: usize,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(format!("{}{key}:", INDENT_STR.repeat(indent_count)));
    result.extend(_render_unchanged_hashmap_value(value, indent_count + 1));
    result
}

fn _render_unchanged(key: &str, value: &tf::RawValue, indent_count: usize) -> Vec<String> {
    match value {
        tf::RawValue::Object(map) => _render_unchanged_hashmap(key, map, indent_count),
        _ => _render_unchanged_plaintext(key, value, indent_count),
    }
}

fn _render_changed_plaintext(
    key: &str,
    before_value: &tf::RawValue,
    after_value: &tf::RawValue,
    indent_count: usize,
) -> Vec<String> {
    vec![format!(
        "{}{key}: {} -> {}",
        INDENT_STR.repeat(indent_count),
        _render_plaintext(before_value),
        _render_plaintext(after_value)
    )]
}

fn _render_changed_hashmap_value(
    before: &std::collections::HashMap<String, tf::RawValue>,
    after: &std::collections::HashMap<String, tf::RawValue>,
    indent_count: usize,
) -> Vec<String> {
    let mut keys: HashSet<String> = HashSet::new();
    keys.extend(before.keys().cloned());
    keys.extend(after.keys().cloned());

    let mut result: Vec<String> = Vec::new();
    for key in keys.iter().sorted() {
        let before_value = before.get(key).unwrap_or(&tf::RawValue::Null);
        let after_value = after.get(key).unwrap_or(&tf::RawValue::Null);
        result.extend(_render_changed(
            key,
            before_value,
            after_value,
            indent_count,
        ));
    }
    result
}

fn _render_changed_hashmap(
    key: &str,
    before: &std::collections::HashMap<String, tf::RawValue>,
    after: &std::collections::HashMap<String, tf::RawValue>,
    indent_count: usize,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(format!("{}{key}:", INDENT_STR.repeat(indent_count)));
    result.extend(_render_changed_hashmap_value(
        before,
        after,
        indent_count + 1,
    ));
    result
}

fn _render_changed(
    key: &str,
    before_value: &tf::RawValue,
    after_value: &tf::RawValue,
    indent_count: usize,
) -> Vec<String> {
    match (before_value, after_value) {
        (tf::RawValue::Object(before), tf::RawValue::Object(after)) => {
            _render_changed_hashmap(key, before, after, indent_count)
        }
        (tf::RawValue::Null, tf::RawValue::Object(after)) => {
            _render_unchanged_hashmap(key, after, indent_count)
        }
        (tf::RawValue::Object(before), tf::RawValue::Null) => {
            _render_unchanged_hashmap(key, before, indent_count)
        }
        (_, _) => {
            if before_value == after_value {
                _render_unchanged_plaintext(key, before_value, indent_count)
            } else {
                _render_changed_plaintext(key, before_value, after_value, indent_count)
            }
        }
    }
}

fn render_changes(args: &Args) -> tera::Result<tera::Value> {
    let raw_change = args.get("change").ok_or("change must be present in args")?;
    let change = tera::from_value::<tf::RawResourceChangeChange>(raw_change.clone())?;

    let before = change.before;
    let after = change.after;

    match (before, after) {
        (Some(before), Some(after)) => {
            let result = _render_changed_hashmap_value(&before, &after, 0);
            Ok(tera::Value::String(result.join("\n")))
        }
        (Some(before), None) => {
            let result = _render_unchanged_hashmap_value(&before, 0);
            Ok(tera::Value::String(result.join("\n")))
        }
        (None, Some(after)) => {
            let result = _render_unchanged_hashmap_value(&after, 0);
            Ok(tera::Value::String(result.join("\n")))
        }
        (None, None) => Ok(tera::Value::String(String::new())),
    }
}

/// # Errors
/// Returns an error if template is invalid or rendering fails
pub fn render(data: &tf::Data, template: &str) -> Result<String, types::Error> {
    let mut tera = tera::Tera::default();
    tera.register_function("render_changes", render_changes);
    tera.register_function("render_action", render_action);
    tera.register_function("render_actions", render_actions);

    let template_name = "template";
    match tera.add_raw_template(template_name, template) {
        Ok(()) => {}
        Err(e) => {
            return Err(types::Error::inherit(
                &e,
                &format!("Failed to add template({template})"),
            ));
        }
    }

    let mut context = tera::Context::new();
    context.insert("data", &data);

    match tera.render(template_name, &context) {
        Ok(result) => Ok(result),
        Err(e) => Err(types::Error::inherit(
            &e,
            &format!("Failed to render template({template})"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod render_action {
        use super::*;

        fn test_with_context(context: tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_action", render_action);

            tera.add_raw_template("template", "{{ render_action(action=action) }}")
                .unwrap();

            tera.render("template", &context)
        }

        fn test(action: tf::Action) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("action", &action);

            test_with_context(context)
        }

        #[test]
        fn options() {
            assert_eq!(test(tf::Action::Create).unwrap(), "✅");
            assert_eq!(test(tf::Action::Delete).unwrap(), "❌");
            assert_eq!(test(tf::Action::DeleteCreate).unwrap(), "♻️");
            assert_eq!(test(tf::Action::Update).unwrap(), "🔄");
            assert_eq!(test(tf::Action::NoOp).unwrap(), "🟰");
            assert_eq!(test(tf::Action::Read).unwrap(), "🔍");
            assert_eq!(test(tf::Action::Unknown).unwrap(), "❓");
        }

        #[test]
        fn not_in_args() {
            let context = tera::Context::new();
            let mut tera = tera::Tera::default();
            tera.register_function("render_action", render_action);
            tera.add_raw_template("template", "{{ render_action() }}")
                .unwrap();

            tera.render("template", &context).unwrap_err();
        }

        #[test]
        fn invalid_action() {
            let action = "invalid".to_string();
            let mut context = tera::Context::new();
            context.insert("action", &action);

            test_with_context(context).unwrap_err();
        }
    }

    mod render_actions {
        use super::*;

        fn test_with_context(context: tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_actions", render_actions);

            tera.add_raw_template("template", "{{ render_actions(actions=actions) }}")
                .unwrap();

            tera.render("template", &context)
        }

        fn test(actions: Vec<tf::Action>) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("actions", &actions);

            test_with_context(context)
        }

        #[test]
        fn default() {
            let actions = vec![
                tf::Action::Create,
                tf::Action::Delete,
                tf::Action::DeleteCreate,
                tf::Action::Update,
                tf::Action::NoOp,
                tf::Action::Read,
                tf::Action::Unknown,
            ];
            assert_eq!(test(actions).unwrap(), "✅❌♻\u{fe0f}🔄🟰🔍❓");
        }

        #[test]
        fn no_actions() {
            let actions = vec![];
            assert_eq!(test(actions).unwrap(), "");
        }

        #[test]
        fn not_in_args() {
            let context = tera::Context::new();
            let mut tera = tera::Tera::default();
            tera.register_function("render_actions", render_actions);
            tera.add_raw_template("template", "{{ render_actions() }}")
                .unwrap();

            tera.render("template", &context).unwrap_err();
        }

        #[test]
        fn invalid_actions() {
            let actions = "invalid".to_string();
            let mut context = tera::Context::new();
            context.insert("actions", &actions);

            test_with_context(context).unwrap_err();
        }
    }

    mod render_changes {
        use super::*;

        fn test_with_context(context: tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_changes", render_changes);

            tera.add_raw_template("template", "{{ render_changes(change=change) }}")
                .unwrap();

            tera.render("template", &context)
        }

        fn test(before: Option<tf::ValueMap>, after: Option<tf::ValueMap>) -> tera::Result<String> {
            let change = tf::RawResourceChangeChange {
                actions: vec![tf::RawResourceChangeChangeAction::Create],
                before,
                after,
                before_sensitive: None,
                after_sensitive: None,
            };

            let mut context = tera::Context::new();
            context.insert("change", &change);

            test_with_context(context)
        }

        fn get_test_data() -> tf::ValueMap {
            let mut map = std::collections::HashMap::new();
            map.insert(
                "string".to_string(),
                tf::RawValue::String("string".to_string()),
            );
            map.insert("integer".to_string(), tf::RawValue::Integer(42));
            map.insert("float".to_string(), tf::RawValue::Float(42.1));
            map.insert("bool".to_string(), tf::RawValue::Boolean(true));
            map.insert(
                "array".to_string(),
                tf::RawValue::Array(vec![tf::RawValue::Integer(42)]),
            );
            map.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::RawValue::Integer(42));
                tf::RawValue::Object(map)
            });
            map.insert("object_to_null".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::RawValue::Integer(42));
                tf::RawValue::Object(map)
            });
            map.insert("null_to_object".to_string(), tf::RawValue::Null);
            map.insert("null".to_string(), tf::RawValue::Null);
            map
        }

        fn get_another_test_data() -> tf::ValueMap {
            let mut map = std::collections::HashMap::new();
            map.insert(
                "string".to_string(),
                tf::RawValue::String("another string".to_string()),
            );
            map.insert("integer".to_string(), tf::RawValue::Integer(43));
            map.insert("float".to_string(), tf::RawValue::Float(43.1));
            map.insert("bool".to_string(), tf::RawValue::Boolean(false));
            map.insert(
                "array".to_string(),
                tf::RawValue::Array(vec![tf::RawValue::Integer(43)]),
            );
            map.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::RawValue::Integer(43));
                tf::RawValue::Object(map)
            });
            map.insert("object_to_null".to_string(), tf::RawValue::Null);
            map.insert("null_to_object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::RawValue::Integer(43));
                tf::RawValue::Object(map)
            });
            map.insert("null".to_string(), tf::RawValue::Null);
            map
        }

        #[test]
        fn before_after() {
            let result = test(Some(get_test_data()), Some(get_another_test_data())).unwrap();

            let expected = r#"array: [42] -> [43]
bool: true -> false
float: 42.1 -> 43.1
integer: 42 -> 43
null: null
null_to_object:
  inner_integer: 43
object:
  inner_integer: 42 -> 43
object_to_null:
  inner_integer: 42
string: "string" -> "another string""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn before() {
            let result = test(Some(get_test_data()), None).unwrap();

            let expected = r#"array: [42]
bool: true
float: 42.1
integer: 42
null: null
null_to_object: null
object:
  inner_integer: 42
object_to_null:
  inner_integer: 42
string: "string""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn after() {
            let result = test(None, Some(get_test_data())).unwrap();

            let expected = r#"array: [42]
bool: true
float: 42.1
integer: 42
null: null
null_to_object: null
object:
  inner_integer: 42
object_to_null:
  inner_integer: 42
string: "string""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn none() {
            let result = test(None, None).unwrap();
            assert_eq!(result, "");
        }

        #[test]
        fn not_in_args() {
            let context = tera::Context::new();
            let mut tera = tera::Tera::default();
            tera.register_function("render_changes", render_changes);
            tera.add_raw_template("template", "{{ render_changes() }}")
                .unwrap();

            tera.render("template", &context).unwrap_err();
        }

        #[test]
        fn invalid_change() {
            let change = "invalid".to_string();
            let mut context = tera::Context::new();
            context.insert("change", &change);

            test_with_context(context).unwrap_err();
        }
    }

    mod render {
        use super::*;
        use crate::utils;

        #[test]
        fn default() {
            let data = tf::tests::get_test_data();
            let result = render(&data, GITHUB_MARKDOWN_TEMPLATE).unwrap();

            let expected =
                utils::test::get_test_data_file_contents("renders/tera/github_markdown.md");
            pretty_assertions::assert_eq!(expected, result);
        }

        #[test]
        fn invalid_render() {
            let data = tf::tests::get_test_data();
            let result = render(&data, "{{ incorrect_data }}").unwrap_err();

            assert_eq!(
                result.message,
                "Failed to render template({{ incorrect_data }}). Failed to render 'template'"
            );
        }
    }
}
