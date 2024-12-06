use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

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
<summary>{{ render_action(action=change.action) }}{{ change.address }}
</summary>

```
{{ render_values(before=change.before, after=change.after, show_changed_values=options.show_changed_values) }}
```

</details>
{%- endfor %}
{%- endif %}
</details>
{% endfor %}";
const DEFAULT_SHOW_CHANGED_VALUES: bool = true;

type Args = std::collections::HashMap<String, tera::Value>;

fn render_action(action: &tf::Action) -> String {
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

fn tera_render_action(args: &Args) -> tera::Result<tera::Value> {
    let action = args.get("action").ok_or("action must be present in args")?;
    let action = tera::from_value::<tf::Action>(action.clone())?;

    Ok(tera::Value::String(render_action(&action)))
}

fn tera_render_actions(args: &Args) -> tera::Result<tera::Value> {
    let actions = args
        .get("actions")
        .ok_or("actions must be present in args")?;
    let actions = tera::from_value::<Vec<tf::Action>>(actions.clone())?;

    let result: Vec<String> = actions.iter().map(render_action).collect();

    Ok(tera::Value::String(result.join("")))
}

fn render_plaintext(value: &tf::Value) -> String {
    match value {
        tf::Value::Sensitive => "sensitive".to_string(),
        tf::Value::String(value) => format!("{}", tera::to_value(value).unwrap()),
        tf::Value::Integer(value) => format!("{}", tera::to_value(value).unwrap()),
        tf::Value::Float(value) => format!("{}", tera::to_value(value).unwrap()),
        tf::Value::Boolean(value) => format!("{}", tera::to_value(value).unwrap()),
        tf::Value::Array(value) => format!(
            "[{}]",
            value
                .iter()
                .map(render_plaintext)
                .collect::<Vec<String>>()
                .join(", ")
        ),
        tf::Value::Object(value) => format!("{}", tera::to_value(value).unwrap()),
        tf::Value::Null => "null".to_string(),
    }
}

fn render_unchanged_plaintext(key: &str, value: &tf::Value, indent_count: usize) -> Vec<String> {
    vec![format!(
        "{}{key}: {}",
        INDENT_STR.repeat(indent_count),
        render_plaintext(value)
    )]
}

fn render_unchanged_hashmap_value(
    value: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for (key, value) in value.iter().sorted_by_key(|x| x.0) {
        result.extend(render_unchanged(key, value, indent_count));
    }
    result
}

fn render_unchanged_hashmap(
    key: &str,
    value: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(format!("{}{key}:", INDENT_STR.repeat(indent_count)));
    result.extend(render_unchanged_hashmap_value(value, indent_count + 1));
    result
}

fn render_unchanged(key: &str, value: &tf::Value, indent_count: usize) -> Vec<String> {
    match value {
        tf::Value::Object(map) => render_unchanged_hashmap(key, map, indent_count),
        _ => render_unchanged_plaintext(key, value, indent_count),
    }
}

fn render_changed_plaintext(
    key: &str,
    before_value: &tf::Value,
    after_value: &tf::Value,
    indent_count: usize,
) -> Vec<String> {
    vec![format!(
        "{}{key}: {} -> {}",
        INDENT_STR.repeat(indent_count),
        render_plaintext(before_value),
        render_plaintext(after_value)
    )]
}

fn render_changed_hashmap_value(
    before: &std::collections::HashMap<String, tf::Value>,
    after: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
    show_changed_values: bool,
) -> Vec<String> {
    let mut keys: HashSet<String> = HashSet::new();
    keys.extend(before.keys().cloned());
    keys.extend(after.keys().cloned());

    let mut result: Vec<String> = Vec::new();
    for key in keys.iter().sorted() {
        let before_value = before.get(key).unwrap_or(&tf::Value::Null);
        let after_value = after.get(key).unwrap_or(&tf::Value::Null);
        result.extend(render_changed(
            key,
            before_value,
            after_value,
            indent_count,
            show_changed_values,
        ));
    }
    result
}

fn render_changed_hashmap(
    key: &str,
    before: &std::collections::HashMap<String, tf::Value>,
    after: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
    show_changed_values: bool,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(format!("{}{key}:", INDENT_STR.repeat(indent_count)));
    result.extend(render_changed_hashmap_value(
        before,
        after,
        indent_count + 1,
        show_changed_values,
    ));
    result
}

fn render_changed(
    key: &str,
    before_value: &tf::Value,
    after_value: &tf::Value,
    indent_count: usize,
    show_changed_values: bool,
) -> Vec<String> {
    match (before_value, after_value) {
        (tf::Value::Object(before), tf::Value::Object(after)) => {
            render_changed_hashmap(key, before, after, indent_count, show_changed_values)
        }
        (tf::Value::Null, tf::Value::Object(after)) => {
            render_unchanged_hashmap(key, after, indent_count)
        }
        (tf::Value::Object(before), tf::Value::Null) => {
            render_unchanged_hashmap(key, before, indent_count)
        }
        (_, _) => {
            if before_value != after_value {
                render_changed_plaintext(key, before_value, after_value, indent_count)
            } else if show_changed_values {
                render_unchanged_plaintext(key, before_value, indent_count)
            } else {
                Vec::new()
            }
        }
    }
}

fn tera_render_values(args: &Args) -> tera::Result<tera::Value> {
    let before = args.get("before").ok_or("before must be present in args")?;
    let after = args.get("after").ok_or("after must be present in args")?;
    let show_changed_values = args
        .get("show_changed_values")
        .unwrap_or(&tera::Value::Bool(DEFAULT_SHOW_CHANGED_VALUES));

    let before = tera::from_value::<Option<tf::ValueMap>>(before.clone())?;
    let after = tera::from_value::<Option<tf::ValueMap>>(after.clone())?;
    let show_changed_values = tera::from_value::<bool>(show_changed_values.clone())?;

    match (before, after, show_changed_values) {
        (Some(before), Some(after), _) => {
            let result = render_changed_hashmap_value(&before, &after, 0, show_changed_values);
            Ok(tera::Value::String(result.join("\n")))
        }
        (Some(before), None, _) => {
            let result = render_unchanged_hashmap_value(&before, 0);
            Ok(tera::Value::String(result.join("\n")))
        }
        (None, Some(after), _) => {
            let result = render_unchanged_hashmap_value(&after, 0);
            Ok(tera::Value::String(result.join("\n")))
        }
        _ => Ok(tera::Value::String(String::new())),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RenderOptionValue {
    Bool(bool),
    String(String),
}

pub type RenderOptions = std::collections::HashMap<String, RenderOptionValue>;

/// # Errors
/// Returns an error if template is invalid or rendering fails
pub fn render(
    data: &tf::Data,
    template: &str,
    options: Option<RenderOptions>,
) -> Result<String, types::Error> {
    let mut tera = tera::Tera::default();
    tera.register_function("render_values", tera_render_values);
    tera.register_function("render_action", tera_render_action);
    tera.register_function("render_actions", tera_render_actions);

    let template_name = "template";
    match tera.add_raw_template(template_name, template) {
        Ok(()) => {}
        Err(e) => {
            return Err(types::Error::chain(
                format!("Failed to add template({template})"),
                e,
            ));
        }
    }

    let mut context = tera::Context::new();
    context.insert("data", &data);
    let options = options.unwrap_or_default();
    context.insert("options", &options);

    match tera.render(template_name, &context) {
        Ok(result) => Ok(result),
        Err(e) => Err(types::Error::chain(
            "Failed to render template".to_string(),
            e,
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
            tera.register_function("render_action", tera_render_action);

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
            tera.register_function("render_action", tera_render_action);
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
            tera.register_function("render_actions", tera_render_actions);

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
            tera.register_function("render_actions", tera_render_actions);
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

    mod render_values {
        use super::*;

        fn test_with_context(context: tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_values", tera_render_values);

            tera.add_raw_template(
                "template",
                "{{ render_values(before=before, after=after) }}",
            )
            .unwrap();

            tera.render("template", &context)
        }

        fn test(before: Option<tf::ValueMap>, after: Option<tf::ValueMap>) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("before", &before);
            context.insert("after", &after);

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
            tera.register_function("render_values", tera_render_values);
            tera.add_raw_template("template", "{{ render_values() }}")
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
            let mut options = RenderOptions::new();
            options.insert(
                "show_changed_values".to_string(),
                RenderOptionValue::Bool(false),
            );
            let result = render(&data, GITHUB_MARKDOWN_TEMPLATE, Some(options)).unwrap();

            let expected =
                utils::test::get_test_data_file_contents("tera/renders/github_markdown/default.md");
            pretty_assertions::assert_eq!(expected, result);
        }

        #[test]
        fn invalid_render() {
            let data = tf::tests::get_test_data();
            let result = render(&data, "{{ incorrect_data }}", None).unwrap_err();

            assert_eq!(
                result.full_message(),
                "Failed to render template. Failed to render 'template'. Variable `incorrect_data` not found in context while rendering 'template'"
            );
        }
    }
}
