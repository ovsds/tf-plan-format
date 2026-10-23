use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use crate::tf;
use crate::types;
use core::str;
use std::collections::HashSet;

const INDENT_STR: &str = "  ";
const MAX_ARRAY_DIFF_CELLS: usize = 4_000_000;

macro_rules! collapsed_values {
    () => {
        " Values not shown to fit the length limit"
    };
}

/// The diff line the github template renders in place of collapsed values
pub const COLLAPSED_VALUES: &str = collapsed_values!();

/// Starts with a hidden marker, so a caller can find and update the pull request comment holding the render.
///
/// Renders the plans with changes first and those without last, in one block: the order dropping the last plans and
/// the plan indices of the collapsing options follow. A plan's changes and outputs share one `diff` code block, a
/// header line each
pub const GITHUB_MARKDOWN_TEMPLATE: &str = concat!("
{%- set plan_count = options.plan_count | default(value=data.plans | length) %}
{%- set collapsed_outputs = options.collapsed_outputs | default(value=[]) -%}
<!-- tf-plan-format -->
{% set_global plan_index = 0 %}
{%- for plan_key, plan in data.plans %}
{%- set changes = plan.changes | filter_actions(exclude=options.hidden_actions) %}
{%- if changes or plan.outputs.actions %}
{%- if plan_index < plan_count %}
{%- set collapsed_changes = options.collapsed_changes[plan_index] | default(value=[]) %}
{%- set plan_actions = render_actions(actions=changes | map(attribute=\"action\") | concat(with=plan.outputs.actions)) %}<details>
<summary>{{ plan_actions }}{% if plan_actions %} {% endif %}<b>{{ plan.title }}</b> +{{ plan.summary.add }} ~{{ plan.summary.change }} −{{ plan.summary.destroy }}</summary>

```diff
{%- for change in changes %}
@@ {{ render_action(action=change.action) }} {{ change.address }} @@
{%- if loop.index0 in collapsed_changes %}
", collapsed_values!(), "
{%- else %}
{%- set values = render_diff_values(before=change.before, after=change.after, show_changed_values=options.show_changed_values) %}
{%- if values %}
{{ values }}
{%- endif %}{% endif %}
{%- endfor %}
{%- if plan.outputs.actions %}
@@ {{ render_actions(actions=plan.outputs.actions) }} Outputs @@
{%- if plan_index in collapsed_outputs %}
", collapsed_values!(), "
{%- else %}
{%- set values = render_diff_values(before=plan.outputs.before, after=plan.outputs.after, show_changed_values=options.show_changed_values) %}
{%- if values %}
{{ values }}
{%- endif %}{% endif %}
{%- endif %}
```

</details>
{% endif %}
{%- set_global plan_index = plan_index + 1 %}
{%- endif %}
{%- endfor %}
{%- set_global unchanged_titles = [] %}
{%- for plan_key, plan in data.plans %}
{%- set changes = plan.changes | filter_actions(exclude=options.hidden_actions) %}
{%- if not changes and not plan.outputs.actions %}
{%- if plan_index < plan_count %}
{%- set_global unchanged_titles = unchanged_titles | concat(with=plan.title) %}
{%- endif %}
{%- set_global plan_index = plan_index + 1 %}
{%- endif %}
{%- endfor %}
{%- if unchanged_titles %}<details>
<summary>💤 No changes ({{ unchanged_titles | length }})</summary>
<blockquote>
{%- for title in unchanged_titles %}
<div>💤 <b>{{ title }}</b></div>
{%- endfor %}
</blockquote>
</details>
{% endif %}
{%- if options.note is defined %}{% if plan_count > 0 %}
{% endif %}_{{ options.note }}_
{% endif %}");
const DEFAULT_SHOW_CHANGED_VALUES: bool = true;

type Args = std::collections::HashMap<String, tera::Value>;

fn render_action(action: &tf::Action) -> String {
    match action {
        tf::Action::Create => "✅".to_string(),
        tf::Action::Delete => "❌".to_string(),
        tf::Action::DeleteCreate => "♻️".to_string(),
        tf::Action::Update => "🔄".to_string(),
        tf::Action::NoOp => "💤".to_string(),
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
    let mut actions = tera::from_value::<Vec<tf::Action>>(actions.clone())?;
    actions.sort();
    actions.dedup();

    let result: Vec<String> = actions.iter().map(render_action).collect();

    Ok(tera::Value::String(result.join("")))
}

fn tera_filter_actions(value: &tera::Value, args: &Args) -> tera::Result<tera::Value> {
    let (actions, include) = match (args.get("include"), args.get("exclude")) {
        (Some(actions), None) => (actions, true),
        (None, Some(actions)) => (actions, false),
        _ => return Err("exactly one of include or exclude must be present in args".into()),
    };
    let actions = tera::from_value::<Vec<tf::Action>>(actions.clone())?;
    let items = tera::from_value::<Vec<tera::Value>>(value.clone())?;

    let mut result: Vec<tera::Value> = Vec::new();
    for item in items {
        let action = item
            .get("action")
            .ok_or("action must be present in items")?;
        let action = tera::from_value::<tf::Action>(action.clone())?;
        if keeps_action(&action, &actions, include) {
            result.push(item);
        }
    }

    Ok(tera::Value::Array(result))
}

/// The `filter_actions` rule, shared so code can find the items the template renders
#[must_use]
pub fn keeps_action(action: &tf::Action, actions: &[tf::Action], include: bool) -> bool {
    actions.contains(action) == include
}

fn render_plaintext(value: &tf::Value) -> String {
    match value {
        tf::Value::Sensitive => "sensitive".to_string(),
        tf::Value::SensitiveChanged => "sensitive (changed)".to_string(),
        tf::Value::SensitiveUnknown => "sensitive (known after apply)".to_string(),
        tf::Value::Unknown => "(known after apply)".to_string(),
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
        tf::Value::Object(value) => format!(
            "{{{}}}",
            value
                .iter()
                .sorted_by_key(|x| x.0)
                .map(|(key, value)| format!("{key}: {}", render_plaintext(value)))
                .collect::<Vec<String>>()
                .join(", ")
        ),
        tf::Value::Null => "null".to_string(),
    }
}

/// How a values line shows in a `diff` code block
#[derive(Clone, Copy)]
enum LineKind {
    Same,
    Changed,
    Added,
    Removed,
}

impl LineKind {
    fn prefix(self) -> char {
        match self {
            LineKind::Same => ' ',
            LineKind::Changed => '!',
            LineKind::Added => '+',
            LineKind::Removed => '-',
        }
    }
}

/// A rendered values line
struct ValueLine {
    text: String,
    kind: LineKind,
}

impl ValueLine {
    fn unchanged(text: String) -> Self {
        ValueLine {
            text,
            kind: LineKind::Same,
        }
    }

    fn changed(text: String) -> Self {
        ValueLine {
            text,
            kind: LineKind::Changed,
        }
    }
}

fn mark(mut lines: Vec<ValueLine>, kind: LineKind) -> Vec<ValueLine> {
    for line in &mut lines {
        line.kind = kind;
    }
    lines
}

/// The line of a key whose value is rendered on the lines below it
fn key_line(key: &str, indent_count: usize) -> ValueLine {
    ValueLine::unchanged(format!("{}{key}:", INDENT_STR.repeat(indent_count)))
}

fn render_unchanged_plaintext(key: &str, value: &tf::Value, indent_count: usize) -> Vec<ValueLine> {
    vec![ValueLine::unchanged(format!(
        "{}{key}: {}",
        INDENT_STR.repeat(indent_count),
        render_plaintext(value)
    ))]
}

fn render_unchanged_hashmap_value(
    value: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
) -> Vec<ValueLine> {
    let mut result: Vec<ValueLine> = Vec::new();
    for (key, value) in value.iter().sorted_by_key(|x| x.0) {
        result.extend(render_unchanged(key, value, indent_count));
    }
    result
}

fn render_unchanged_hashmap(
    key: &str,
    value: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
) -> Vec<ValueLine> {
    let mut result = vec![key_line(key, indent_count)];
    result.extend(render_unchanged_hashmap_value(value, indent_count + 1));
    result
}

fn render_unchanged(key: &str, value: &tf::Value, indent_count: usize) -> Vec<ValueLine> {
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
) -> Vec<ValueLine> {
    vec![ValueLine::changed(format!(
        "{}{key}: {} -> {}",
        INDENT_STR.repeat(indent_count),
        render_plaintext(before_value),
        render_plaintext(after_value)
    ))]
}

fn render_changed_hashmap_value(
    before: &std::collections::HashMap<String, tf::Value>,
    after: &std::collections::HashMap<String, tf::Value>,
    indent_count: usize,
    show_changed_values: bool,
) -> Vec<ValueLine> {
    let mut keys: HashSet<String> = HashSet::new();
    keys.extend(before.keys().cloned());
    keys.extend(after.keys().cloned());

    let mut result: Vec<ValueLine> = Vec::new();
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
) -> Vec<ValueLine> {
    // Without unchanged values an unchanged object has nothing under its key
    if before == after && !show_changed_values {
        return Vec::new();
    }
    let mut result = vec![key_line(key, indent_count)];
    result.extend(render_changed_hashmap_value(
        before,
        after,
        indent_count + 1,
        show_changed_values,
    ));
    result
}

enum ArrayChange<'a> {
    Same(&'a tf::Value),
    Changed(&'a tf::Value, &'a tf::Value),
}

fn diff_arrays<'a>(before: &'a [tf::Value], after: &'a [tf::Value]) -> Vec<ArrayChange<'a>> {
    let prefix = before
        .iter()
        .zip(after)
        .take_while(|(before, after)| before == after)
        .count();
    let suffix = before[prefix..]
        .iter()
        .rev()
        .zip(after[prefix..].iter().rev())
        .take_while(|(before, after)| before == after)
        .count();

    let mut result: Vec<ArrayChange> = before[..prefix].iter().map(ArrayChange::Same).collect();
    result.extend(diff_arrays_middle(
        &before[prefix..before.len() - suffix],
        &after[prefix..after.len() - suffix],
    ));
    result.extend(
        before[before.len() - suffix..]
            .iter()
            .map(ArrayChange::Same),
    );
    result
}

/// Pairs elements by their longest common subsequence, so an insertion or removal does not shift every later element
/// into a change; removed and added elements between two common ones are paired up in order, the rest diff with null.
fn diff_arrays_middle<'a>(before: &'a [tf::Value], after: &'a [tf::Value]) -> Vec<ArrayChange<'a>> {
    // The table is quadratic: past the cap, elements pair by position
    if before.len().saturating_mul(after.len()) > MAX_ARRAY_DIFF_CELLS {
        return (0..before.len().max(after.len()))
            .map(|index| match (before.get(index), after.get(index)) {
                (Some(before), Some(after)) if before == after => ArrayChange::Same(before),
                (before, after) => ArrayChange::Changed(
                    before.unwrap_or(&tf::Value::Null),
                    after.unwrap_or(&tf::Value::Null),
                ),
            })
            .collect();
    }

    let mut common = vec![vec![0usize; after.len() + 1]; before.len() + 1];
    for i in (0..before.len()).rev() {
        for j in (0..after.len()).rev() {
            common[i][j] = if before[i] == after[j] {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }

    let mut result: Vec<ArrayChange> = Vec::new();
    let mut removed: Vec<&tf::Value> = Vec::new();
    let mut added: Vec<&tf::Value> = Vec::new();
    let flush = |result: &mut Vec<ArrayChange<'a>>,
                 removed: &mut Vec<&'a tf::Value>,
                 added: &mut Vec<&'a tf::Value>| {
        for index in 0..removed.len().max(added.len()) {
            let before_value = removed.get(index).copied().unwrap_or(&tf::Value::Null);
            let after_value = added.get(index).copied().unwrap_or(&tf::Value::Null);
            result.push(ArrayChange::Changed(before_value, after_value));
        }
        removed.clear();
        added.clear();
    };

    let (mut i, mut j) = (0, 0);
    while i < before.len() || j < after.len() {
        if i < before.len() && j < after.len() && before[i] == after[j] {
            flush(&mut result, &mut removed, &mut added);
            result.push(ArrayChange::Same(&before[i]));
            i += 1;
            j += 1;
        } else if j == after.len() || (i < before.len() && common[i + 1][j] >= common[i][j + 1]) {
            removed.push(&before[i]);
            i += 1;
        } else {
            added.push(&after[j]);
            j += 1;
        }
    }
    flush(&mut result, &mut removed, &mut added);
    result
}

/// Turns lines rendered one level deeper into a list item: the first line's last indent becomes the `- ` marker.
/// Nested renderers print nothing for an empty object or one whose only difference is a key turned null: the
/// inline `fallback` keeps the element visible.
fn render_array_item(
    mut lines: Vec<ValueLine>,
    indent_count: usize,
    mut fallback: ValueLine,
) -> Vec<ValueLine> {
    let item_indent = format!("{}- ", INDENT_STR.repeat(indent_count));
    let Some(first) = lines.first_mut() else {
        fallback.text.insert_str(0, &item_indent);
        return vec![fallback];
    };
    let nested_indent = INDENT_STR.repeat(indent_count + 1);
    if first.text.starts_with(&nested_indent) {
        first
            .text
            .replace_range(..nested_indent.len(), &item_indent);
    }
    lines
}

fn render_changed_inline(before: &tf::Value, after: &tf::Value) -> String {
    format!(
        "{} -> {}",
        render_plaintext(before),
        render_plaintext(after)
    )
}

fn render_changed_array_value(
    before: &[tf::Value],
    after: &[tf::Value],
    indent_count: usize,
    show_changed_values: bool,
) -> Vec<ValueLine> {
    let indent = INDENT_STR.repeat(indent_count);
    let hidden_marker = format!("{indent}- ...");
    let mut result: Vec<ValueLine> = Vec::new();
    for change in diff_arrays(before, after) {
        match change {
            // A run of hidden elements keeps one marker, so a change still shows where in the array it is
            ArrayChange::Same(_) if !show_changed_values => {
                if result.last().map(|line| &line.text) != Some(&hidden_marker) {
                    result.push(ValueLine::unchanged(hidden_marker.clone()));
                }
            }
            ArrayChange::Same(value @ tf::Value::Object(map)) => {
                result.extend(render_array_item(
                    render_unchanged_hashmap_value(map, indent_count + 1),
                    indent_count,
                    ValueLine::unchanged(render_plaintext(value)),
                ));
            }
            ArrayChange::Same(value) => {
                result.push(ValueLine::unchanged(format!(
                    "{indent}- {}",
                    render_plaintext(value)
                )));
            }
            ArrayChange::Changed(
                before_value @ tf::Value::Object(before),
                after_value @ tf::Value::Object(after),
            ) => {
                result.extend(render_array_item(
                    render_changed_hashmap_value(
                        before,
                        after,
                        indent_count + 1,
                        show_changed_values,
                    ),
                    indent_count,
                    ValueLine::changed(render_changed_inline(before_value, after_value)),
                ));
            }
            ArrayChange::Changed(
                before_value @ tf::Value::Array(before),
                after_value @ tf::Value::Array(after),
            ) => {
                result.extend(render_array_item(
                    render_changed_array_value(
                        before,
                        after,
                        indent_count + 1,
                        show_changed_values,
                    ),
                    indent_count,
                    ValueLine::changed(render_changed_inline(before_value, after_value)),
                ));
            }
            ArrayChange::Changed(before, after) => {
                result.push(ValueLine::changed(format!(
                    "{indent}- {}",
                    render_changed_inline(before, after)
                )));
            }
        }
    }
    result
}

fn render_changed(
    key: &str,
    before_value: &tf::Value,
    after_value: &tf::Value,
    indent_count: usize,
    show_changed_values: bool,
) -> Vec<ValueLine> {
    match (before_value, after_value) {
        (tf::Value::Object(before), tf::Value::Object(after)) => {
            render_changed_hashmap(key, before, after, indent_count, show_changed_values)
        }
        (tf::Value::Array(before), tf::Value::Array(after)) if before != after => {
            let mut result = vec![key_line(key, indent_count)];
            result.extend(render_changed_array_value(
                before,
                after,
                indent_count + 1,
                show_changed_values,
            ));
            result
        }
        // The whole object appeared or disappeared under its key, its key line included
        (tf::Value::Null, tf::Value::Object(after)) => mark(
            render_unchanged_hashmap(key, after, indent_count),
            LineKind::Changed,
        ),
        (tf::Value::Object(before), tf::Value::Null) => mark(
            render_unchanged_hashmap(key, before, indent_count),
            LineKind::Changed,
        ),
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

type RenderValuesFn = fn(Option<&tf::ValueMap>, Option<&tf::ValueMap>, bool) -> String;

/// Calls `render` with the `before`, `after` and optional `show_changed_values` arguments of a tera function
fn tera_values(args: &Args, render: RenderValuesFn) -> tera::Result<tera::Value> {
    let before = args.get("before").ok_or("before must be present in args")?;
    let after = args.get("after").ok_or("after must be present in args")?;
    let show_changed_values = args
        .get("show_changed_values")
        .unwrap_or(&tera::Value::Bool(DEFAULT_SHOW_CHANGED_VALUES));

    let before = tera::from_value::<Option<tf::ValueMap>>(before.clone())?;
    let after = tera::from_value::<Option<tf::ValueMap>>(after.clone())?;
    let show_changed_values = tera::from_value::<bool>(show_changed_values.clone())?;

    Ok(tera::Value::String(render(
        before.as_ref(),
        after.as_ref(),
        show_changed_values,
    )))
}

fn tera_render_values(args: &Args) -> tera::Result<tera::Value> {
    tera_values(args, render_values)
}

fn tera_render_diff_values(args: &Args) -> tera::Result<tera::Value> {
    tera_values(args, render_diff_values)
}

fn render_value_lines(
    before: Option<&tf::ValueMap>,
    after: Option<&tf::ValueMap>,
    show_changed_values: bool,
) -> Vec<ValueLine> {
    match (before, after) {
        (Some(before), Some(after)) => {
            render_changed_hashmap_value(before, after, 0, show_changed_values)
        }
        (Some(before), None) => mark(render_unchanged_hashmap_value(before, 0), LineKind::Removed),
        (None, Some(after)) => mark(render_unchanged_hashmap_value(after, 0), LineKind::Added),
        (None, None) => Vec::new(),
    }
}

#[must_use]
pub fn render_values(
    before: Option<&tf::ValueMap>,
    after: Option<&tf::ValueMap>,
    show_changed_values: bool,
) -> String {
    render_value_lines(before, after, show_changed_values)
        .into_iter()
        .map(|line| line.text)
        .join("\n")
}

/// The `render_values` lines, each prefixed for a `diff` code block: `+` when only `after` exists, `-` when only
/// `before` does, and with both `!` for a line carrying a change and a space for the others
#[must_use]
pub fn render_diff_values(
    before: Option<&tf::ValueMap>,
    after: Option<&tf::ValueMap>,
    show_changed_values: bool,
) -> String {
    render_value_lines(before, after, show_changed_values)
        .iter()
        .format_with("\n", |line, f| {
            f(&format_args!("{} {}", line.kind.prefix(), line.text))
        })
        .to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RenderOptionValue {
    Bool(bool),
    String(String),
    Actions(Vec<tf::Action>),
    Count(usize),
    Indices(Vec<usize>),
    IndexLists(Vec<Vec<usize>>),
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
    tera.register_function("render_diff_values", tera_render_diff_values);
    tera.register_function("render_action", tera_render_action);
    tera.register_function("render_actions", tera_render_actions);
    tera.register_filter("filter_actions", tera_filter_actions);

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

    fn strings(values: &[&str]) -> Vec<tf::Value> {
        values
            .iter()
            .map(|value| tf::Value::String(value.to_string()))
            .collect()
    }

    fn values(entries: Vec<(&str, tf::Value)>) -> tf::ValueMap {
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    fn object(entries: Vec<(&str, tf::Value)>) -> tf::Value {
        tf::Value::Object(values(entries))
    }

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
            assert_eq!(test(tf::Action::NoOp).unwrap(), "💤");
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
            assert_eq!(test(actions).unwrap(), "✅♻\u{fe0f}🔍🔄❌💤❓");
        }

        #[test]
        fn duplicates() {
            let actions = vec![
                tf::Action::Update,
                tf::Action::Create,
                tf::Action::Update,
                tf::Action::Create,
            ];
            assert_eq!(test(actions).unwrap(), "✅🔄");
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

    mod filter_actions {
        use super::*;

        fn test(template: &str) -> tera::Result<String> {
            test_with_changes(
                template,
                vec![
                    serde_json::json!({"address": "create", "action": "Create"}),
                    serde_json::json!({"address": "no_op", "action": "NoOp"}),
                    serde_json::json!({"address": "read", "action": "Read"}),
                    serde_json::json!({"address": "update", "action": "Update"}),
                ],
            )
        }

        fn test_with_changes(template: &str, changes: Vec<tera::Value>) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_filter("filter_actions", tera_filter_actions);
            tera.add_raw_template("template", template).unwrap();

            let mut context = tera::Context::new();
            context.insert("changes", &changes);
            context.insert("actions", &vec![tf::Action::NoOp, tf::Action::Read]);
            tera.render("template", &context)
        }

        #[test]
        fn include() {
            let result = test(
                "{% for change in changes | filter_actions(include=actions) %}{{ change.address }} {% endfor %}",
            );
            assert_eq!(result.unwrap(), "no_op read ");
        }

        #[test]
        fn exclude() {
            let result = test(
                "{% for change in changes | filter_actions(exclude=actions) %}{{ change.address }} {% endfor %}",
            );
            assert_eq!(result.unwrap(), "create update ");
        }

        #[test]
        fn exclude_none() {
            let result = test(
                "{% for change in changes | filter_actions(exclude=[]) %}{{ change.address }} {% endfor %}",
            );
            assert_eq!(result.unwrap(), "create no_op read update ");
        }

        fn assert_args_error(result: tera::Result<String>) {
            let error = result.unwrap_err();
            let mut source: &dyn std::error::Error = &error;
            while let Some(next) = source.source() {
                source = next;
            }
            assert_eq!(
                source.to_string(),
                "exactly one of include or exclude must be present in args"
            );
        }

        #[test]
        fn both() {
            assert_args_error(test(
                "{{ changes | filter_actions(include=actions, exclude=actions) }}",
            ));
        }

        #[test]
        fn neither() {
            assert_args_error(test("{{ changes | filter_actions() }}"));
        }

        #[test]
        fn invalid_actions() {
            test("{{ changes | filter_actions(include=[\"invalid\"]) }}").unwrap_err();
        }

        #[test]
        fn invalid_value() {
            test("{{ \"invalid\" | filter_actions(include=actions) }}").unwrap_err();
        }

        #[test]
        fn no_action() {
            let changes = vec![serde_json::json!({"address": "no_action"})];
            test_with_changes("{{ changes | filter_actions(include=actions) }}", changes)
                .unwrap_err();
        }

        #[test]
        fn invalid_item_action() {
            let changes = vec![serde_json::json!({"action": "invalid"})];
            test_with_changes("{{ changes | filter_actions(include=actions) }}", changes)
                .unwrap_err();
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

            let expected = r#"array:
  - 42 -> 43
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

        fn get_unknown_before() -> tf::ValueMap {
            let mut map = std::collections::HashMap::new();
            map.insert("string".to_string(), tf::Value::String("abc".to_string()));
            map.insert("sensitive".to_string(), tf::Value::Sensitive);
            map.insert("same".to_string(), tf::Value::Integer(42));
            map.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner".to_string(), tf::Value::Integer(42));
                tf::Value::Object(map)
            });
            map
        }

        fn get_unknown_after() -> tf::ValueMap {
            let mut map = std::collections::HashMap::new();
            map.insert("string".to_string(), tf::Value::Unknown);
            map.insert("sensitive".to_string(), tf::Value::SensitiveUnknown);
            map.insert("same".to_string(), tf::Value::Integer(42));
            map.insert("new".to_string(), tf::Value::Unknown);
            map.insert(
                "array".to_string(),
                tf::Value::Array(vec![tf::Value::Integer(42), tf::Value::Unknown]),
            );
            map.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert("inner".to_string(), tf::Value::Unknown);
                tf::Value::Object(map)
            });
            map
        }

        fn test_show_changed_values(
            before: Option<tf::ValueMap>,
            after: Option<tf::ValueMap>,
            show_changed_values: bool,
        ) -> String {
            let mut tera = tera::Tera::default();
            tera.register_function("render_values", tera_render_values);
            tera.add_raw_template(
                "template",
                "{{ render_values(before=before, after=after, show_changed_values=show_changed_values) }}",
            )
            .unwrap();

            let mut context = tera::Context::new();
            context.insert("before", &before);
            context.insert("after", &after);
            context.insert("show_changed_values", &show_changed_values);
            tera.render("template", &context).unwrap()
        }

        #[test]
        fn after_unknown() {
            let result = test(None, Some(get_unknown_after())).unwrap();

            let expected = r#"array: [42, (known after apply)]
new: (known after apply)
object:
  inner: (known after apply)
same: 42
sensitive: sensitive (known after apply)
string: (known after apply)"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn before_after_unknown() {
            let result = test_show_changed_values(
                Some(get_unknown_before()),
                Some(get_unknown_after()),
                true,
            );

            let expected = r#"array: null -> [42, (known after apply)]
new: null -> (known after apply)
object:
  inner: 42 -> (known after apply)
same: 42
sensitive: sensitive -> sensitive (known after apply)
string: "abc" -> (known after apply)"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn before_after_unknown_hide_unchanged() {
            let result = test_show_changed_values(
                Some(get_unknown_before()),
                Some(get_unknown_after()),
                false,
            );

            let expected = r#"array: null -> [42, (known after apply)]
new: null -> (known after apply)
object:
  inner: 42 -> (known after apply)
sensitive: sensitive -> sensitive (known after apply)
string: "abc" -> (known after apply)"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn object_unchanged_hide_unchanged() {
            let object = tf::Value::Object(std::collections::HashMap::from([(
                "env".to_string(),
                tf::Value::String("prod".to_string()),
            )]));
            let values = std::collections::HashMap::from([("tags".to_string(), object)]);

            let hidden =
                test_show_changed_values(Some(values.clone()), Some(values.clone()), false);
            pretty_assertions::assert_eq!(hidden, "");

            let shown = test_show_changed_values(Some(values.clone()), Some(values), true);
            pretty_assertions::assert_eq!(shown, "tags:\n  env: \"prod\"");
        }

        fn test_array(
            before: Vec<tf::Value>,
            after: Vec<tf::Value>,
            show_changed_values: bool,
        ) -> String {
            let mut before_map = std::collections::HashMap::new();
            before_map.insert("array".to_string(), tf::Value::Array(before));
            let mut after_map = std::collections::HashMap::new();
            after_map.insert("array".to_string(), tf::Value::Array(after));
            test_show_changed_values(Some(before_map), Some(after_map), show_changed_values)
        }

        #[test]
        fn array_insert() {
            let result = test_array(
                strings(&["a", "b", "c"]),
                strings(&["a", "x", "b", "c"]),
                true,
            );

            let expected = r#"array:
  - "a"
  - null -> "x"
  - "b"
  - "c""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_remove_hide_unchanged() {
            let result = test_array(strings(&["a", "b", "c"]), strings(&["a", "c"]), false);

            let expected = r#"array:
  - ...
  - "b" -> null
  - ..."#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_replace_and_append() {
            let result = test_array(strings(&["a", "b"]), strings(&["a", "x", "y"]), false);

            let expected = r#"array:
  - ...
  - "b" -> "x"
  - null -> "y""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_unknown_element() {
            let result = test_array(
                strings(&["a", "b"]),
                vec![tf::Value::String("a".to_string()), tf::Value::Unknown],
                false,
            );

            let expected = r#"array:
  - ...
  - "b" -> (known after apply)"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_objects() {
            let before = vec![
                object(vec![
                    ("port", tf::Value::Integer(80)),
                    ("name", tf::Value::String("http".to_string())),
                ]),
                object(vec![("port", tf::Value::Integer(22))]),
            ];
            let after = vec![
                object(vec![
                    ("port", tf::Value::Integer(443)),
                    ("name", tf::Value::String("http".to_string())),
                ]),
                object(vec![("port", tf::Value::Integer(22))]),
            ];

            let result = test_array(before.clone(), after.clone(), true);
            let expected = r#"array:
  - name: "http"
    port: 80 -> 443
  - port: 22"#;
            pretty_assertions::assert_eq!(result, expected);

            let result = test_array(before, after, false);
            let expected = r#"array:
  - port: 80 -> 443
  - ..."#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_nested() {
            let result = test_array(
                vec![tf::Value::Array(strings(&["a", "b"]))],
                vec![tf::Value::Array(strings(&["a", "c"]))],
                false,
            );

            let expected = r#"array:
  - - ...
    - "b" -> "c""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_empty_object() {
            let result = test_array(
                vec![object(vec![]), tf::Value::String("a".to_string())],
                vec![object(vec![]), tf::Value::String("b".to_string())],
                true,
            );

            let expected = r#"array:
  - {}
  - "a" -> "b""#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_object_key_to_null() {
            let result = test_array(
                vec![object(vec![
                    ("a", tf::Value::Integer(1)),
                    ("b", tf::Value::Null),
                ])],
                vec![object(vec![("a", tf::Value::Integer(1))])],
                false,
            );

            let expected = r#"array:
  - {a: 1, b: null} -> {a: 1}"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_large_append() {
            let before: Vec<tf::Value> = (0..100_000).map(tf::Value::Integer).collect();
            let mut after = before.clone();
            after.push(tf::Value::Integer(-1));

            let result = test_array(before, after, false);

            let expected = r#"array:
  - ...
  - null -> -1"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_large_both_ends_changed() {
            let before: Vec<tf::Value> = (0..3_000).map(tf::Value::Integer).collect();
            let mut after = before.clone();
            after[0] = tf::Value::Integer(-1);
            after[2_999] = tf::Value::Integer(-2);

            let result = test_array(before, after, false);

            let expected = r#"array:
  - 0 -> -1
  - ...
  - 2999 -> -2"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn array_unchanged() {
            let result = test_array(strings(&["a"]), strings(&["a"]), true);
            pretty_assertions::assert_eq!(result, r#"array: ["a"]"#);

            let result = test_array(strings(&["a"]), strings(&["a"]), false);
            pretty_assertions::assert_eq!(result, "");
        }

        #[test]
        fn object_to_unknown() {
            let mut before = std::collections::HashMap::new();
            before.insert("object".to_string(), {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "b".to_string(),
                    tf::Value::Array(vec![tf::Value::Integer(1)]),
                );
                map.insert("a".to_string(), tf::Value::String("x".to_string()));
                tf::Value::Object(map)
            });
            let mut after = std::collections::HashMap::new();
            after.insert("object".to_string(), tf::Value::Unknown);

            let result = test_show_changed_values(Some(before), Some(after), false);

            let expected = r#"object: {a: "x", b: [1]} -> (known after apply)"#;
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

    mod render_diff_values {
        use super::*;

        fn render_template(template: &str, context: &tera::Context) -> tera::Result<String> {
            let mut tera = tera::Tera::default();
            tera.register_function("render_diff_values", tera_render_diff_values);
            tera.add_raw_template("template", template).unwrap();
            tera.render("template", context)
        }

        fn test(
            before: Option<tf::ValueMap>,
            after: Option<tf::ValueMap>,
            show_changed_values: bool,
        ) -> String {
            let mut context = tera::Context::new();
            context.insert("before", &before);
            context.insert("after", &after);
            context.insert("show_changed_values", &show_changed_values);
            render_template(
                "{{ render_diff_values(before=before, after=after, show_changed_values=show_changed_values) }}",
                &context,
            )
            .unwrap()
        }

        fn get_values() -> tf::ValueMap {
            values(vec![
                ("string", tf::Value::String("foo".to_string())),
                ("object", object(vec![("inner", tf::Value::Integer(42))])),
            ])
        }

        fn get_before() -> tf::ValueMap {
            values(vec![
                ("changed", tf::Value::Integer(1)),
                ("same", tf::Value::Integer(1)),
                ("gone", object(vec![("a", tf::Value::Integer(1))])),
                ("appeared", tf::Value::Null),
                (
                    "object",
                    object(vec![
                        ("inner", tf::Value::Integer(1)),
                        ("kept", tf::Value::String("k".to_string())),
                    ]),
                ),
            ])
        }

        fn get_after() -> tf::ValueMap {
            values(vec![
                ("changed", tf::Value::Integer(2)),
                ("same", tf::Value::Integer(1)),
                ("gone", tf::Value::Null),
                ("appeared", object(vec![("b", tf::Value::Integer(2))])),
                (
                    "object",
                    object(vec![
                        ("inner", tf::Value::Integer(2)),
                        ("kept", tf::Value::String("k".to_string())),
                    ]),
                ),
            ])
        }

        #[test]
        fn after_only() {
            let result = test(None, Some(get_values()), false);

            let expected = "+ object:\n+   inner: 42\n+ string: \"foo\"";
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn before_only() {
            let result = test(Some(get_values()), None, true);

            let expected = "- object:\n-   inner: 42\n- string: \"foo\"";
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn before_after() {
            let result = test(Some(get_before()), Some(get_after()), true);

            let expected = r#"! appeared:
!   b: 2
! changed: 1 -> 2
! gone:
!   a: 1
  object:
!   inner: 1 -> 2
    kept: "k"
  same: 1"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn before_after_hide_unchanged() {
            let result = test(Some(get_before()), Some(get_after()), false);

            let expected = r#"! appeared:
!   b: 2
! changed: 1 -> 2
! gone:
!   a: 1
  object:
!   inner: 1 -> 2"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        fn get_array_before() -> tf::ValueMap {
            values(vec![
                ("names", tf::Value::Array(strings(&["a", "b"]))),
                (
                    "ports",
                    tf::Value::Array(vec![object(vec![
                        ("name", tf::Value::String("http".to_string())),
                        ("port", tf::Value::Integer(80)),
                    ])]),
                ),
                (
                    "nested",
                    tf::Value::Array(vec![
                        object(vec![("tags", object(vec![("env", tf::Value::Integer(1))]))]),
                        tf::Value::Array(vec![tf::Value::Integer(1), tf::Value::Integer(2)]),
                        object(vec![]),
                        object(vec![("a", tf::Value::Integer(1)), ("b", tf::Value::Null)]),
                    ]),
                ),
            ])
        }

        fn get_array_after() -> tf::ValueMap {
            values(vec![
                ("names", tf::Value::Array(strings(&["a", "c"]))),
                (
                    "ports",
                    tf::Value::Array(vec![object(vec![
                        ("name", tf::Value::String("http".to_string())),
                        ("port", tf::Value::Integer(443)),
                    ])]),
                ),
                (
                    "nested",
                    tf::Value::Array(vec![
                        object(vec![("tags", object(vec![("env", tf::Value::Integer(2))]))]),
                        tf::Value::Array(vec![tf::Value::Integer(1), tf::Value::Integer(3)]),
                        object(vec![]),
                        object(vec![("a", tf::Value::Integer(1))]),
                    ]),
                ),
            ])
        }

        #[test]
        fn arrays() {
            let result = test(Some(get_array_before()), Some(get_array_after()), true);

            let expected = r#"  names:
    - "a"
!   - "b" -> "c"
  nested:
    - tags:
!       env: 1 -> 2
    - - 1
!     - 2 -> 3
    - {}
    - a: 1
      b: null
  ports:
    - name: "http"
!     port: 80 -> 443"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn arrays_hide_unchanged() {
            let result = test(Some(get_array_before()), Some(get_array_after()), false);

            let expected = r#"  names:
    - ...
!   - "b" -> "c"
  nested:
    - tags:
!       env: 1 -> 2
    - - ...
!     - 2 -> 3
    - ...
!   - {a: 1, b: null} -> {a: 1}
  ports:
!   - port: 80 -> 443"#;
            pretty_assertions::assert_eq!(result, expected);
        }

        #[test]
        fn same_lines_as_render_values() {
            for (before, after) in [
                (Some(get_before()), Some(get_after())),
                (Some(get_array_before()), Some(get_array_after())),
                (None, Some(get_values())),
                (Some(get_values()), None),
            ] {
                for show_changed_values in [false, true] {
                    let diff =
                        render_diff_values(before.as_ref(), after.as_ref(), show_changed_values);
                    let values =
                        render_values(before.as_ref(), after.as_ref(), show_changed_values);

                    let unprefixed = diff.lines().map(|line| &line[2..]).join("\n");
                    pretty_assertions::assert_eq!(unprefixed, values);
                }
            }
        }

        #[test]
        fn none() {
            assert_eq!(test(None, None, true), "");
        }

        #[test]
        fn unchanged_hide_unchanged() {
            assert_eq!(test(Some(get_values()), Some(get_values()), false), "");
        }

        #[test]
        fn not_in_args() {
            render_template("{{ render_diff_values() }}", &tera::Context::new()).unwrap_err();
        }

        #[test]
        fn default_show_changed_values() {
            let mut context = tera::Context::new();
            context.insert("before", &Some(get_before()));
            context.insert("after", &Some(get_after()));
            let result = render_template(
                "{{ render_diff_values(before=before, after=after) }}",
                &context,
            )
            .unwrap();

            pretty_assertions::assert_eq!(
                result,
                test(Some(get_before()), Some(get_after()), true)
            );
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
            options.insert(
                "hidden_actions".to_string(),
                RenderOptionValue::Actions(vec![tf::Action::NoOp, tf::Action::Read]),
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
