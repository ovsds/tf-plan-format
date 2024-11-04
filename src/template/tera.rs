use crate::tf;
use crate::types;
use core::str;
use std::collections::HashSet;

pub const GITHUB_MARKDOWN_TEMPLATE: &str = "
{%- for plan_key, plan in data.plans %}<details>
<summary>{{ render_plan_actions(plan=plan) }}{{ plan_key }}</summary>
{%- if plan.resource_changes %}
{%- for resource_change in plan.resource_changes %}
<details>
<summary>{{ render_actions(actions=resource_change.change.actions) }}{{ resource_change.address }}
</summary>

```
{{ render_changes(change=resource_change.change) }}
```

</details>
{%- endfor %}
{%- else %}
No resource changes
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

fn render_changes(args: &Args) -> tera::Result<tera::Value> {
    let raw_change = args.get("change").ok_or("change must be present in args")?;
    let change = tera::from_value::<tf::ResourceChangeChange>(raw_change.clone())?;

    match (change.before, change.after) {
        (Some(before), Some(after)) => {
            let mut result: Vec<String> = Vec::new();

            for (key, value) in &before {
                match after.get(key) {
                    Some(after_value) => {
                        if value == after_value {
                            result.push(format!("{key}: {}", tera::to_value(value)?));
                        } else {
                            result.push(format!(
                                "{key}: {} -> {}",
                                tera::to_value(value)?,
                                tera::to_value(after_value)?
                            ));
                        }
                    }
                    None => {
                        result.push(format!("{key}: {} -> null", tera::to_value(value)?));
                    }
                }
            }

            for (key, value) in after {
                if !before.contains_key(&key) {
                    result.push(format!("{key}: null -> {}", tera::to_value(value)?));
                }
            }
            result.sort();
            Ok(tera::Value::String(result.join("\n")))
        }
        (Some(before), None) => {
            let mut result: Vec<String> = Vec::new();
            for (key, value) in before {
                result.push(format!("{key}: {}", tera::to_value(value)?));
            }
            result.sort();
            Ok(tera::Value::String(result.join("\n")))
        }
        (None, Some(after)) => {
            let mut result: Vec<String> = Vec::new();
            for (key, value) in after {
                result.push(format!("{key}: {}", tera::to_value(value)?));
            }
            result.sort();
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
            };

            let mut context = tera::Context::new();
            context.insert("change", &change);

            test_with_context(context)
        }

        #[test]
        fn default() {
            let mut before: tf::ValueMap = std::collections::HashMap::new();
            before.insert("key".to_string(), Some(tf::Value::Integer(42.into())));
            let mut after: tf::ValueMap = std::collections::HashMap::new();
            after.insert("key".to_string(), Some(tf::Value::Integer(43.into())));

            assert_eq!(test(Some(before), Some(after)).unwrap(), "key: 42 -> 43");
        }

        #[test]
        fn no_changes() {
            let mut before: tf::ValueMap = std::collections::HashMap::new();
            before.insert("key".to_string(), Some(tf::Value::Integer(42.into())));
            let after = before.clone();

            assert_eq!(test(Some(before), Some(after)).unwrap(), "key: 42");
        }

        #[test]
        fn no_before_value() {
            let before: tf::ValueMap = std::collections::HashMap::new();
            let mut after: tf::ValueMap = std::collections::HashMap::new();
            after.insert("key".to_string(), Some(tf::Value::Integer(42.into())));

            assert_eq!(test(Some(before), Some(after)).unwrap(), "key: null -> 42");
        }

        #[test]
        fn no_after_value() {
            let mut before: tf::ValueMap = std::collections::HashMap::new();
            before.insert("key".to_string(), Some(tf::Value::Integer(42.into())));
            let after: tf::ValueMap = std::collections::HashMap::new();

            assert_eq!(test(Some(before), Some(after)).unwrap(), "key: 42 -> null");
        }

        #[test]
        fn no_before() {
            let mut after: tf::ValueMap = std::collections::HashMap::new();
            after.insert("key".to_string(), Some(tf::Value::Integer(42.into())));

            assert_eq!(test(None, Some(after)).unwrap(), "key: 42");
        }

        #[test]
        fn no_after() {
            let mut before: tf::ValueMap = std::collections::HashMap::new();
            before.insert("key".to_string(), Some(tf::Value::Integer(42.into())));

            assert_eq!(test(Some(before), None).unwrap(), "key: 42");
        }

        #[test]
        fn no_before_no_after() {
            assert_eq!(test(None, None).unwrap(), "");
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
