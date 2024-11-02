use crate::tf;
use crate::types;
use core::str;
use std::collections::HashSet;

pub const GITHUB_MARKDOWN_TEMPLATE: &str = "
{%- for plan_key, plan in data.plans %}<details>
<summary>{{ plan_key }}</summary>
{%- if plan.resource_changes %}
{%- for resource_change in plan.resource_changes %}
<details>
<summary>{{ render_actions(actions=resource_change.change.actions) }}{{ resource_change.address }}
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

type Args = std::collections::HashMap<String, tera::Value>;

fn render_result_action(action: &tf::ResultAction) -> String {
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
    let Some(tera::Value::Array(actions)) = args.get("actions") else {
        return Err("actions must be an array".into());
    };

    let actions: Vec<String> = actions
        .iter()
        .map(|action| match action {
            tera::Value::String(action) => Ok(action.clone()),
            _ => Err("actions must be a string".into()),
        })
        .collect::<tera::Result<Vec<String>>>()?;

    match tf::ResultAction::from_strings(&actions) {
        Ok(result_action) => Ok(tera::Value::String(render_result_action(&result_action))),
        Err(e) => Err(e.to_string().into()),
    }
}

fn render_changes(args: &Args) -> tera::Result<tera::Value> {
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
    tera.register_function("render_actions", render_actions);

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

    mod render_actions {
        use super::*;

        fn test(actions: impl serde::Serialize) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("actions", &actions);

            let mut tera = tera::Tera::default();
            tera.register_function("render_actions", render_actions);

            tera.add_raw_template("template", "{{ render_actions(actions=actions) }}")
                .unwrap();

            tera.render("template", &context)
        }

        #[test]
        fn create() {
            let actions = vec![tf::ResourceChangeChangeAction::Create];
            assert_eq!(test(&actions).unwrap(), "✅");
        }

        #[test]
        fn delete() {
            let actions = vec![tf::ResourceChangeChangeAction::Delete];
            assert_eq!(test(&actions).unwrap(), "❌");
        }

        #[test]
        fn delete_create() {
            let actions = vec![
                tf::ResourceChangeChangeAction::Delete,
                tf::ResourceChangeChangeAction::Create,
            ];
            assert_eq!(test(&actions).unwrap(), "♻️");
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
            assert_eq!(test(&actions).unwrap(), "🔄");
        }

        #[test]
        fn no_op() {
            let actions = vec![tf::ResourceChangeChangeAction::NoOp];
            assert_eq!(test(&actions).unwrap(), "🟰");
        }

        #[test]
        fn read() {
            let actions = vec![tf::ResourceChangeChangeAction::Read];
            assert_eq!(test(&actions).unwrap(), "🔍");
        }

        #[test]
        fn unknown() {
            let actions: Vec<String> = vec![];
            assert_eq!(test(&actions).unwrap(), "❓");
        }

        #[test]
        fn not_array() {
            let actions = "not an array";
            test(&actions).unwrap_err();
        }

        #[test]
        fn not_string() {
            let actions = vec![1];
            test(&actions).unwrap_err();
        }

        #[test]
        fn invalid_action() {
            let actions = vec!["invalid"];
            test(&actions).unwrap_err();
        }
    }

    mod render_changes {
        use super::*;

        fn test(
            before: impl serde::Serialize,
            after: impl serde::Serialize,
        ) -> tera::Result<String> {
            let mut context = tera::Context::new();
            context.insert("before", &before);
            context.insert("after", &after);

            let mut tera = tera::Tera::default();
            tera.register_function("render_changes", render_changes);

            tera.add_raw_template(
                "template",
                "{{ render_changes(before=before, after=after) }}",
            )
            .unwrap();

            tera.render("template", &context)
        }

        #[test]
        fn default() {
            let mut before = std::collections::HashMap::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));
            let mut after = std::collections::HashMap::new();
            after.insert("key".to_string(), tera::Value::Number(43.into()));

            assert_eq!(test(&before, &after).unwrap(), "key: 42 -> 43\n");
        }

        #[test]
        fn no_changes() {
            let mut before = std::collections::HashMap::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));
            let after = before.clone();

            assert_eq!(test(&before, &after).unwrap(), "key: 42\n");
        }

        #[test]
        fn no_before_key() {
            let before: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let mut after = std::collections::HashMap::new();
            after.insert("key".to_string(), tera::Value::Number(42.into()));

            assert_eq!(test(&before, &after).unwrap(), "key: null -> 42\n");
        }

        #[test]
        fn no_after_key() {
            let mut before = std::collections::HashMap::new();
            before.insert("key".to_string(), tera::Value::Number(42.into()));
            let after: std::collections::HashMap<String, String> = std::collections::HashMap::new();

            assert_eq!(test(&before, &after).unwrap(), "key: 42 -> null\n");
        }

        #[test]
        fn null_before() {
            let before = tera::Value::Null;
            let mut after = std::collections::HashMap::new();
            after.insert("key".to_string(), 42);

            assert_eq!(test(&before, &after).unwrap(), "key: 42\n");
        }

        #[test]
        fn null_after() {
            let mut before = std::collections::HashMap::new();
            before.insert("key".to_string(), 42);
            let after = tera::Value::Null;

            assert_eq!(test(&before, &after).unwrap(), "key: 42\n");
        }

        #[test]
        fn null_before_after() {
            let before = tera::Value::Null;
            let after = tera::Value::Null;

            assert_eq!(test(&before, &after).unwrap(), "");
        }

        #[test]
        fn before_wrong_type() {
            let before = "not an object";
            let after: std::collections::HashMap<String, String> = std::collections::HashMap::new();

            test(&before, &after).unwrap_err();
        }

        #[test]
        fn after_wrong_type() {
            let before: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let after = "not an object";

            test(&before, &after).unwrap_err();
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
