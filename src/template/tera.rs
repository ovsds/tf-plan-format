use crate::tf;
use crate::types;
use core::str;

pub const GITHUB_MARKDOWN_TEMPLATE: &str = "
{%- for plan_key, plan in data.plans %}<details>
<summary>{{ plan_key }}</summary>
{%- if plan.resource_changes %}
{%- for resource_change in plan.resource_changes %}
<details>
<summary>
{%- if resource_change.change.actions is containing('create') and resource_change.change.actions is containing('delete') %}♻️
{%- elif resource_change.change.actions is containing('create') %}✅
{%- elif resource_change.change.actions is containing('delete') %}❌
{%- elif resource_change.change.actions is containing('update') %}🔄
{%- elif resource_change.change.actions is containing('no-op') %}🟰
{%- elif resource_change.change.actions is containing('read') %}🔍
{%- else %}❓
{%- endif -%}
{{ resource_change.address }}
</summary>

```
{{ render_changes(before=resource_change.change.before, after=resource_change.change.after) -}}
```

</details>
{%- endfor %}
{%- else %}
No resource changes
{%- endif %}
</details>
{% endfor %}";

fn render_changes(
    args: &std::collections::HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    match (args.get("before"), args.get("after")) {
        (Some(tera::Value::Object(before)), Some(tera::Value::Object(after))) => {
            let mut result: Vec<(String, String)> = Vec::new();

            for (key, value) in before {
                match after.get(key) {
                    Some(after_value) => {
                        if value == after_value {
                            result.push((key.clone(), format!("{value}")));
                        } else {
                            result.push((key.clone(), format!("{value} -> {after_value}")));
                        }
                    }
                    None => {
                        result.push((key.clone(), format!("{value} -> null")));
                    }
                }
            }

            for (key, value) in after {
                if !before.contains_key(key) {
                    result.push((key.clone(), format!("null -> {value}")));
                }
            }
            result.sort();

            let mut result_str = String::new();
            for (key, value) in result {
                result_str.push_str(&format!("{key}: {value}\n"));
            }
            Ok(tera::Value::String(result_str))
        }
        (Some(tera::Value::Object(before)), Some(tera::Value::Null)) => {
            let mut result = String::new();
            for (key, value) in before {
                result.push_str(&format!("{key}: {value}\n"));
            }
            Ok(tera::Value::String(result))
        }
        (Some(tera::Value::Null), Some(tera::Value::Object(after))) => {
            let mut result = String::new();
            for (key, value) in after {
                result.push_str(&format!("{key}: {value}\n"));
            }
            Ok(tera::Value::String(result))
        }
        (Some(tera::Value::Null), Some(tera::Value::Null)) => {
            Ok(tera::Value::String(String::new()))
        }
        _ => Err("before and after must be objects".into()),
    }
}

/// # Errors
/// Returns an error if template is invalid or rendering fails
pub fn render(data: &tf::Data, template: &str) -> Result<String, types::Error> {
    let mut tera = tera::Tera::default();
    tera.register_function("render_changes", render_changes);

    let template_name = "template";
    match tera.add_raw_template(template_name, template) {
        Ok(()) => {}
        Err(e) => {
            return Err(types::Error::inherit(
                e,
                &format!("Failed to add template({template})"),
            ));
        }
    }

    let mut context = tera::Context::new();
    context.insert("data", &data);

    match tera.render(template_name, &context) {
        Ok(result) => Ok(result),
        Err(e) => Err(types::Error::inherit(
            e,
            &format!("Failed to render template({template})"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod render_changes {
        use super::*;

        #[test]
        fn default() {
            let mut before = tera::Map::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));
            let mut after = tera::Map::new();
            after.insert("key".to_string(), tera::Value::Number(43.into()));

            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Object(before));
            args.insert("after".to_string(), tera::Value::Object(after));

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String("key: 42 -> 43\n".to_string()), result);
        }

        #[test]
        fn no_changes() {
            let mut before = tera::Map::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));
            let mut after = tera::Map::new();
            after.insert("key".to_string(), tera::Value::Number(42.into()));

            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Object(before));
            args.insert("after".to_string(), tera::Value::Object(after));

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String("key: 42\n".to_string()), result);
        }

        #[test]
        fn no_before_key() {
            let before = tera::Map::new();
            let mut after = tera::Map::new();
            after.insert("key".to_string(), tera::Value::Number(42.into()));

            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Object(before));
            args.insert("after".to_string(), tera::Value::Object(after));

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String("key: null -> 42\n".to_string()), result);
        }

        #[test]
        fn no_after_key() {
            let mut before = tera::Map::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));
            let after = tera::Map::new();

            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Object(before));
            args.insert("after".to_string(), tera::Value::Object(after));

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String("key: 42 -> null\n".to_string()), result);
        }

        #[test]
        fn null_before() {
            let mut after = tera::Map::new();
            after.insert("key".to_string(), tera::Value::Number(42.into()));

            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Null);
            args.insert("after".to_string(), tera::Value::Object(after));

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String("key: 42\n".to_string()), result);
        }

        #[test]
        fn null_after() {
            let mut before = tera::Map::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));

            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Object(before));
            args.insert("after".to_string(), tera::Value::Null);

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String("key: 42\n".to_string()), result);
        }

        #[test]
        fn null_before_after() {
            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Null);
            args.insert("after".to_string(), tera::Value::Null);

            let result = render_changes(&args).unwrap();
            assert_eq!(tera::Value::String(String::new()), result);
        }

        #[test]
        fn no_before_raises() {
            let mut args = std::collections::HashMap::new();
            args.insert("after".to_string(), tera::Value::Object(tera::Map::new()));

            let result = render_changes(&args);
            assert!(result.is_err());
        }

        #[test]
        fn no_after_raises() {
            let mut args = std::collections::HashMap::new();
            args.insert("before".to_string(), tera::Value::Object(tera::Map::new()));

            let result = render_changes(&args);
            assert!(result.is_err());
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
            assert_eq!(expected, result);
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
