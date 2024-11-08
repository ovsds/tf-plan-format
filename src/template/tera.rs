use itertools::Itertools;

use crate::tf;
use crate::types;
use core::str;
use std::collections::HashSet;

const INDENT_STR: &str = "  ";

pub const GITHUB_MARKDOWN_TEMPLATE: &str = "
{%- for plan_key, plan in data.plans %}<details>
<summary>{{ render_plan_actions(plan=plan) }}{{ plan_key }}</summary>
{%- if not plan.resource_changes %}
No resource changes
{%- else %}
{%- for resource_change in plan.resource_changes %}
<details>
<summary>{{ render_actions(actions=resource_change.change.actions) }}{{ resource_change.address }}
</summary>

```
{{ render_changes(change=resource_change.change) }}
```

</details>
{%- endfor %}
{%- endif %}
</details>
{% endfor %}";

type Args = std::collections::HashMap<String, tera::Value>;

fn _render_result_action(action: &tf::ResultAction) -> String {
    match action {
        tf::ResultAction::Create => "✅".to_string(),
        tf::ResultAction::Delete => "❌".to_string(),
        tf::ResultAction::DeleteCreate => "♻️".to_string(),
        tf::ResultAction::Update => "🔄".to_string(),
        tf::ResultAction::NoOp => "🟰".to_string(),
        tf::ResultAction::Read => "🔍".to_string(),
        tf::ResultAction::Unknown => "❓".to_string(),
    }
}

fn render_actions(args: &Args) -> tera::Result<tera::Value> {
    let raw_actions = args
        .get("actions")
        .ok_or("actions must be present in args")?;
    let actions = tera::from_value::<Vec<tf::ResourceChangeChangeAction>>(raw_actions.clone())?;

    let result_action = tf::ResultAction::from_actions(&actions);
    Ok(tera::Value::String(_render_result_action(&result_action)))
}

fn render_plan_actions(args: &Args) -> tera::Result<tera::Value> {
    let raw_plan = args.get("plan").ok_or("plan must be present in args")?;
    let plan = tera::from_value::<tf::Plan>(raw_plan.clone())?;

    let mut rendered_actions: HashSet<String> = std::collections::HashSet::new();

    for resource_change in plan.resource_changes.unwrap_or_default() {
        let actions = resource_change.change.actions;
        let result_action = tf::ResultAction::from_actions(&actions);

        rendered_actions.insert(_render_result_action(&result_action));
    }
    let mut result: Vec<String> = rendered_actions.into_iter().collect();
    result.sort();

    Ok(tera::Value::String(result.join("")))
}

fn _render_plaintext(value: &tf::Value) -> String {
    format!("{}", tera::to_value(value).unwrap())
}

fn _render_unchanged_plaintext(key: &str, value: &tf::Value, indent_count: usize) -> Vec<String> {
    vec![format!(
        "{}{key}: {}",
        INDENT_STR.repeat(indent_count),
        _render_plaintext(value)
    )]
}

fn _render_unchanged_hashmap_value(
    value: &std::collections::HashMap<String, tf::Value>,
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
    value: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(format!("{}{key}:", INDENT_STR.repeat(indent_count)));
    result.extend(_render_unchanged_hashmap_value(value, indent_count + 1));
    result
}

fn _render_unchanged(key: &str, value: &tf::Value, indent_count: usize) -> Vec<String> {
    match value {
        tf::Value::Object(map) => _render_unchanged_hashmap(key, map, indent_count),
        _ => _render_unchanged_plaintext(key, value, indent_count),
    }
}

fn _render_changed_plaintext(
    key: &str,
    before_value: &tf::Value,
    after_value: &tf::Value,
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
    before: &std::collections::HashMap<String, tf::Value>,
    after: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
) -> Vec<String> {
    let mut keys: HashSet<String> = HashSet::new();
    keys.extend(before.keys().cloned());
    keys.extend(after.keys().cloned());

    let mut result: Vec<String> = Vec::new();
    for key in keys.iter().sorted() {
        let before_value = before.get(key).unwrap_or(&tf::Value::Null);
        let after_value = after.get(key).unwrap_or(&tf::Value::Null);
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
    before: &std::collections::HashMap<String, tf::Value>,
    after: &std::collections::HashMap<String, tf::Value>,
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
    before_value: &tf::Value,
    after_value: &tf::Value,
    indent_count: usize,
) -> Vec<String> {
    match (before_value, after_value) {
        (tf::Value::Object(before), tf::Value::Object(after)) => {
            _render_changed_hashmap(key, before, after, indent_count)
        }
        (tf::Value::Null, tf::Value::Object(after)) => {
            _render_unchanged_hashmap(key, after, indent_count)
        }
        (tf::Value::Object(before), tf::Value::Null) => {
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
    let change = tera::from_value::<tf::ResourceChangeChange>(raw_change.clone())?;

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
    tera.register_function("render_actions", render_actions);
    tera.register_function("render_plan_actions", render_plan_actions);

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

    mod render_actions {
        use super::*;

        fn test_with_context(context: tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_actions", render_actions);

            tera.add_raw_template("template", "{{ render_actions(actions=actions) }}")
                .unwrap();

            tera.render("template", &context)
        }

        fn test(actions: Vec<tf::ResourceChangeChangeAction>) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("actions", &actions);

            test_with_context(context)
        }

        #[test]
        fn create() {
            let actions = vec![tf::ResourceChangeChangeAction::Create];
            assert_eq!(test(actions).unwrap(), "✅");
        }

        #[test]
        fn delete() {
            let actions = vec![tf::ResourceChangeChangeAction::Delete];
            assert_eq!(test(actions).unwrap(), "❌");
        }

        #[test]
        fn delete_create() {
            let actions = vec![
                tf::ResourceChangeChangeAction::Delete,
                tf::ResourceChangeChangeAction::Create,
            ];
            assert_eq!(test(actions).unwrap(), "♻️");
        }

        #[test]
        fn create_delete() {
            let actions = vec![
                tf::ResourceChangeChangeAction::Create,
                tf::ResourceChangeChangeAction::Delete,
            ];
            assert_eq!(test(actions).unwrap(), "♻️");
        }

        #[test]
        fn update() {
            let actions = vec![tf::ResourceChangeChangeAction::Update];
            assert_eq!(test(actions).unwrap(), "🔄");
        }

        #[test]
        fn no_op() {
            let actions = vec![tf::ResourceChangeChangeAction::NoOp];
            assert_eq!(test(actions).unwrap(), "🟰");
        }

        #[test]
        fn read() {
            let actions: Vec<tf::ResourceChangeChangeAction> =
                vec![tf::ResourceChangeChangeAction::Read];
            assert_eq!(test(actions).unwrap(), "🔍");
        }

        #[test]
        fn unknown() {
            let actions: Vec<tf::ResourceChangeChangeAction> = vec![];
            assert_eq!(test(actions).unwrap(), "❓");
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
            let actions = vec!["invalid".to_string()];
            let mut context = tera::Context::new();
            context.insert("actions", &actions);

            test_with_context(context).unwrap_err();
        }
    }

    mod render_plan_actions {
        use super::*;

        fn test_with_context(context: tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_plan_actions", render_plan_actions);

            tera.add_raw_template("template", "{{ render_plan_actions(plan=plan) }}")
                .unwrap();

            tera.render("template", &context)
        }

        fn test(plan: tf::Plan) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("plan", &plan);

            test_with_context(context)
        }

        fn get_resource_change(actions: Vec<tf::ResourceChangeChangeAction>) -> tf::ResourceChange {
            tf::ResourceChange {
                address: "address".to_string(),
                name: "name".to_string(),
                change: tf::ResourceChangeChange {
                    actions,
                    before: None,
                    after: None,
                    before_sensitive: None,
                    after_sensitive: None,
                },
            }
        }

        #[test]
        fn default() {
            let plan = tf::Plan {
                resource_changes: Some(vec![
                    get_resource_change(vec![tf::ResourceChangeChangeAction::Create]),
                    get_resource_change(vec![tf::ResourceChangeChangeAction::Delete]),
                    get_resource_change(vec![
                        tf::ResourceChangeChangeAction::Delete,
                        tf::ResourceChangeChangeAction::Create,
                    ]),
                    get_resource_change(vec![tf::ResourceChangeChangeAction::Update]),
                    get_resource_change(vec![tf::ResourceChangeChangeAction::NoOp]),
                    get_resource_change(vec![tf::ResourceChangeChangeAction::Read]),
                    get_resource_change(vec![]),
                ]),
            };
            assert_eq!(test(plan).unwrap(), "♻\u{fe0f}✅❌❓🔄🔍🟰");
        }

        #[test]
        fn no_resource_changes() {
            let plan = tf::Plan {
                resource_changes: None,
            };
            assert_eq!(test(plan).unwrap(), "");
        }

        #[test]
        fn empty_resource_changes() {
            let plan = tf::Plan {
                resource_changes: Some(vec![]),
            };
            assert_eq!(test(plan).unwrap(), "");
        }

        #[test]
        fn not_in_args() {
            let context = tera::Context::new();
            let mut tera = tera::Tera::default();
            tera.register_function("render_plan_actions", render_plan_actions);
            tera.add_raw_template("template", "{{ render_plan_actions() }}")
                .unwrap();

            tera.render("template", &context).unwrap_err();
        }

        #[test]
        fn invalid_plan() {
            let plan = "invalid".to_string();
            let mut context = tera::Context::new();
            context.insert("plan", &plan);

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
            let change = tf::ResourceChangeChange {
                actions: vec![tf::ResourceChangeChangeAction::Create],
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
                tf::Value::String("string".to_string()),
            );
            map.insert("integer".to_string(), tf::Value::Integer(42));
            map.insert("float".to_string(), tf::Value::Float(42.1));
            map.insert("bool".to_string(), tf::Value::Boolean(true));
            map.insert(
                "array".to_string(),
                tf::Value::Array(vec![tf::Value::Integer(42)]),
            );
            map.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::Value::Integer(42));
                tf::Value::Object(map)
            });
            map.insert("object_to_null".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::Value::Integer(42));
                tf::Value::Object(map)
            });
            map.insert("null_to_object".to_string(), tf::Value::Null);
            map.insert("null".to_string(), tf::Value::Null);
            map
        }

        fn get_another_test_data() -> tf::ValueMap {
            let mut map = std::collections::HashMap::new();
            map.insert(
                "string".to_string(),
                tf::Value::String("another string".to_string()),
            );
            map.insert("integer".to_string(), tf::Value::Integer(43));
            map.insert("float".to_string(), tf::Value::Float(43.1));
            map.insert("bool".to_string(), tf::Value::Boolean(false));
            map.insert(
                "array".to_string(),
                tf::Value::Array(vec![tf::Value::Integer(43)]),
            );
            map.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::Value::Integer(43));
                tf::Value::Object(map)
            });
            map.insert("object_to_null".to_string(), tf::Value::Null);
            map.insert("null_to_object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner_integer".to_string(), tf::Value::Integer(43));
                tf::Value::Object(map)
            });
            map.insert("null".to_string(), tf::Value::Null);
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
