use crate::tf;
use crate::types;
use itertools::Itertools;
use std::collections::HashMap;
use std::str::FromStr;

pub mod tera;

#[derive(Clone, Debug, PartialEq)]
pub enum Engine {
    Tera,
}

impl FromStr for Engine {
    type Err = types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tera" => Ok(Engine::Tera),
            _ => Err(types::Error::default(format!(
                "Invalid template engine: {s}"
            ))),
        }
    }
}

/// # Errors
/// Returns an error if rendering fails
pub fn render(engine: &Engine, data: &tf::Data, template: &str) -> Result<String, types::Error> {
    match engine {
        Engine::Tera => tera::render(data, template, None),
    }
}

/// # Errors
/// Returns an error if rendering fails
pub fn render_github(
    data: &tf::Data,
    show_changed_values: bool,
    show_no_op: bool,
    show_read: bool,
    max_length: Option<usize>,
) -> Result<String, types::Error> {
    let mut hidden_actions = Vec::new();
    if !show_no_op {
        hidden_actions.push(tf::Action::NoOp);
    }
    if !show_read {
        hidden_actions.push(tf::Action::Read);
    }
    let render = |limits: Option<&Limits>| {
        let mut options = tera::RenderOptions::new();
        options.insert(
            "show_changed_values".to_string(),
            tera::RenderOptionValue::Bool(show_changed_values),
        );
        options.insert(
            "hidden_actions".to_string(),
            tera::RenderOptionValue::Actions(hidden_actions.clone()),
        );
        if let Some(limits) = limits {
            limits.insert_options(&mut options);
        }
        tera::render(data, tera::GITHUB_MARKDOWN_TEMPLATE, Some(options))
    };

    let mut result = render(None)?;
    let Some(max_length) = max_length.filter(|&max_length| result.len() > max_length) else {
        return Ok(result);
    };

    let mut limits = Limits {
        max_length,
        plans: get_plan_keys(data),
        ..Limits::default()
    };
    let mut blocks =
        get_values_blocks(data, &limits.plans, &hidden_actions, show_changed_values).into_iter();
    while result.len() > max_length && blocks.len() > 0 {
        // Values length alone estimates the saving, the re-render accounts for the markup and the note
        let excess = result.len() - max_length;
        let mut saving = 0;
        for block in blocks.by_ref() {
            saving += block.length;
            limits.collapse(block);
            if saving >= excess {
                break;
            }
        }
        result = render(Some(&limits))?;
    }
    while result.len() > max_length && limits.drop_last_plan() {
        result = render(Some(&limits))?;
    }
    Ok(result)
}

/// The order Tera iterates plans in: `serde_json` maps are sorted by key
fn get_plan_keys(data: &tf::Data) -> Vec<String> {
    data.plans.keys().sorted().cloned().collect()
}

enum ValuesBlockKind {
    Change(usize),
    Outputs,
}

struct ValuesBlock {
    plan: String,
    kind: ValuesBlockKind,
    length: usize,
}

/// Values blocks the github template renders, largest first, ties in render order
fn get_values_blocks(
    data: &tf::Data,
    plans: &[String],
    hidden_actions: &[tf::Action],
    show_changed_values: bool,
) -> Vec<ValuesBlock> {
    let mut blocks = Vec::new();
    for plan_key in plans {
        let plan = &data.plans[plan_key];
        let changes = plan
            .changes
            .iter()
            .filter(|change| !hidden_actions.contains(&change.action))
            .enumerate()
            .map(|(index, change)| {
                let kind = ValuesBlockKind::Change(index);
                (kind, change.before.as_ref(), change.after.as_ref())
            });
        let outputs = (!plan.outputs.actions.is_empty()).then(|| {
            let outputs = &plan.outputs;
            (
                ValuesBlockKind::Outputs,
                Some(&outputs.before),
                Some(&outputs.after),
            )
        });
        blocks.extend(
            changes
                .chain(outputs)
                .map(|(kind, before, after)| ValuesBlock {
                    plan: plan_key.clone(),
                    kind,
                    length: tera::render_values(before, after, show_changed_values).len(),
                }),
        );
    }
    blocks.sort_by_key(|block| std::cmp::Reverse(block.length));
    blocks
}

#[derive(Default)]
struct Limits {
    max_length: usize,
    plans: Vec<String>,
    dropped_plans: usize,
    collapsed_changes: HashMap<String, Vec<usize>>,
    collapsed_outputs: Vec<String>,
}

impl Limits {
    fn collapse(&mut self, block: ValuesBlock) {
        match block.kind {
            ValuesBlockKind::Change(index) => self
                .collapsed_changes
                .entry(block.plan)
                .or_default()
                .push(index),
            ValuesBlockKind::Outputs => self.collapsed_outputs.push(block.plan),
        }
    }

    fn drop_last_plan(&mut self) -> bool {
        let Some(plan) = self.plans.pop() else {
            return false;
        };
        self.collapsed_changes.remove(&plan);
        self.collapsed_outputs
            .retain(|outputs_plan| *outputs_plan != plan);
        self.dropped_plans += 1;
        true
    }

    fn note(&self) -> String {
        let mut parts = Vec::new();
        match (self.plans.is_empty(), self.dropped_plans) {
            (_, 0) => {}
            (true, 1) => parts.push("the plan".to_string()),
            (true, dropped) => parts.push(format!("all {dropped} plans")),
            (false, 1) => parts.push("the last plan".to_string()),
            (false, dropped) => parts.push(format!("the last {dropped} plans")),
        }
        let collapsed_changes = self.collapsed_changes.values().map(Vec::len).sum();
        if collapsed_changes > 0 {
            parts.push(format!("values of {}", count(collapsed_changes, "change")));
        }
        if !self.collapsed_outputs.is_empty() {
            parts.push(format!(
                "outputs of {}",
                count(self.collapsed_outputs.len(), "plan")
            ));
        }
        let parts = match parts.split_last() {
            Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
            _ => parts.join(""),
        };
        format!(
            "Output limited to {} bytes: {parts} not shown",
            self.max_length
        )
    }

    fn insert_options(&self, options: &mut tera::RenderOptions) {
        options.insert(
            "plans".to_string(),
            tera::RenderOptionValue::Strings(self.plans.clone()),
        );
        options.insert(
            "collapsed_changes".to_string(),
            tera::RenderOptionValue::Indices(self.collapsed_changes.clone()),
        );
        options.insert(
            "collapsed_outputs".to_string(),
            tera::RenderOptionValue::Strings(self.collapsed_outputs.clone()),
        );
        options.insert(
            "note".to_string(),
            tera::RenderOptionValue::String(self.note()),
        );
    }
}

fn count(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod engine_from_str {
        use super::*;

        #[test]
        fn default() {
            assert_eq!(Engine::Tera, "tera".parse().unwrap());
        }

        #[test]
        fn invalid() {
            let result = "invalid".parse::<Engine>();
            assert_eq!(true, result.is_err());
        }
    }

    mod render {
        use super::*;
        use crate::utils;

        #[test]
        fn default() {
            let data = tf::tests::get_test_data();
            let template = utils::test::get_test_data_file_contents("tera/templates/custom");
            let result = render(&Engine::Tera, &data, &template).unwrap();

            let expected = utils::test::get_test_data_file_contents("tera/renders/custom.md");

            pretty_assertions::assert_eq!(expected, result);
        }
    }

    mod render_github {
        use super::*;
        use crate::utils;

        const COLLAPSED: &str = "_Values not shown to fit the length limit_";

        fn get_expected_default() -> String {
            utils::test::get_test_data_file_contents("tera/renders/github_markdown/default.md")
        }

        fn render_capped(data: &tf::Data, max_length: usize) -> String {
            render_github(data, false, false, false, Some(max_length)).unwrap()
        }

        fn get_outputs_data() -> tf::Data {
            let tool = tf::tests::Tool::Terraform;
            let plan_type = tf::tests::PlanType::Outputs;
            tf::Data::new(std::collections::HashMap::from([(
                tf::tests::get_test_plan_file(&tool, &plan_type),
                tf::tests::get_test_plan(&tool, &plan_type),
            )]))
        }

        fn get_plan_section<'a>(result: &'a str, title: &str) -> &'a str {
            let start = result.find(&format!("{title} - ")).unwrap();
            let end = result[start + title.len()..]
                .find(".tfplan.json - ")
                .map_or(result.len(), |end| start + title.len() + end);
            &result[start..end]
        }

        fn get_note(result: &str) -> &str {
            result.lines().last().unwrap()
        }

        #[test]
        fn default() {
            let data = tf::tests::get_test_data();
            let result = render_github(&data, false, false, false, None).unwrap();

            pretty_assertions::assert_eq!(get_expected_default(), result);
        }

        #[test]
        fn cap_not_reached() {
            let data = tf::tests::get_test_data();
            let expected = get_expected_default();

            pretty_assertions::assert_eq!(expected, render_capped(&data, expected.len()));
            pretty_assertions::assert_eq!(expected, render_capped(&data, usize::MAX));
        }

        #[test]
        fn collapses_largest_first() {
            let data = tf::tests::get_test_data();
            let result = render_capped(&data, get_expected_default().len() - 1);

            // Both update-array plans hold the largest values: the tie goes to the one rendered first
            let collapsed = get_plan_section(&result, "update-array/opentofu.tfplan.json");
            assert!(collapsed.contains(&format!(
                "<summary>🔄terraform_data.foo-bar\n</summary>\n\n{COLLAPSED}\n\n</details>"
            )));
            assert!(!collapsed.contains("```"));
            let kept = get_plan_section(&result, "update-array/terraform.tfplan.json");
            assert!(!kept.contains(COLLAPSED));
            assert_eq!(1, result.matches(COLLAPSED).count());
            assert_eq!(
                "_Output limited to 5659 bytes: values of 1 change not shown_",
                get_note(&result)
            );
            assert!(result.ends_with(
                "</details>\n\n_Output limited to 5659 bytes: values of 1 change not shown_\n"
            ));
        }

        #[test]
        fn collapses_outputs() {
            let data = get_outputs_data();
            let full = render_github(&data, false, false, false, None).unwrap();
            let result = render_capped(&data, full.len() - 1);

            assert!(result.contains(&format!(
                "<summary>✅🔄❌Outputs\n</summary>\n\n{COLLAPSED}\n\n</details>"
            )));
            assert!(result.contains("```\ninput: \"foo\" -> \"bar\""));
            assert_eq!(1, result.matches(COLLAPSED).count());
            assert_eq!(
                format!(
                    "_Output limited to {} bytes: outputs of 1 plan not shown_",
                    full.len() - 1
                ),
                get_note(&result)
            );
        }

        #[test]
        fn drops_last_plans() {
            let data = tf::tests::get_test_data();
            let result = render_capped(&data, 2000);

            assert!(result.len() <= 2000);
            assert!(!result.contains("```"));
            assert!(result.contains("no-resources/terraform.tfplan.json - "));
            assert!(!result.contains("outputs/opentofu.tfplan.json - "));
            assert_eq!(
                "_Output limited to 2000 bytes: the last 10 plans and values of 6 changes not shown_",
                get_note(&result)
            );
        }

        #[test]
        fn never_exceeds_cap() {
            let data = tf::tests::get_test_data();
            for show_changed_values in [false, true] {
                let full = render_github(&data, show_changed_values, true, true, None).unwrap();
                for max_length in (100..full.len()).step_by(400) {
                    let result =
                        render_github(&data, show_changed_values, true, true, Some(max_length))
                            .unwrap();

                    assert!(result.len() <= max_length, "{max_length}: {result}");
                    assert_eq!(
                        result.matches("<details>").count(),
                        result.matches("</details>").count(),
                        "{max_length}: {result}"
                    );
                    assert_eq!(
                        0,
                        result.lines().filter(|line| *line == "```").count() % 2,
                        "{max_length}: {result}"
                    );
                }
            }
        }

        #[test]
        fn tiny_cap_keeps_only_note() {
            let data = tf::tests::get_test_data();
            let result = render_capped(&data, 10);

            // Even the note alone exceeds the cap, it is still the best output left
            assert_eq!(
                "_Output limited to 10 bytes: all 20 plans not shown_\n",
                result
            );
        }
    }

    mod limits_note {
        use super::*;

        fn get_note(
            plans: &[&str],
            dropped_plans: usize,
            collapsed_changes: &[(&str, Vec<usize>)],
            collapsed_outputs: &[&str],
        ) -> String {
            let limits = Limits {
                max_length: 65536,
                plans: plans.iter().map(ToString::to_string).collect(),
                dropped_plans,
                collapsed_changes: collapsed_changes
                    .iter()
                    .map(|(plan, indices)| ((*plan).to_string(), indices.clone()))
                    .collect(),
                collapsed_outputs: collapsed_outputs.iter().map(ToString::to_string).collect(),
            };
            limits.note()
        }

        #[test]
        fn changes() {
            assert_eq!(
                "Output limited to 65536 bytes: values of 1 change not shown",
                get_note(&["a"], 0, &[("a", vec![0])], &[])
            );
            assert_eq!(
                "Output limited to 65536 bytes: values of 3 changes not shown",
                get_note(&["a", "b"], 0, &[("a", vec![0, 2]), ("b", vec![1])], &[])
            );
        }

        #[test]
        fn outputs() {
            assert_eq!(
                "Output limited to 65536 bytes: outputs of 2 plans not shown",
                get_note(&["a", "b"], 0, &[], &["a", "b"])
            );
        }

        #[test]
        fn changes_and_outputs() {
            assert_eq!(
                "Output limited to 65536 bytes: values of 2 changes and outputs of 1 plan not shown",
                get_note(&["a"], 0, &[("a", vec![0, 1])], &["a"])
            );
        }

        #[test]
        fn plans() {
            assert_eq!(
                "Output limited to 65536 bytes: the last plan, values of 1 change and outputs of 1 plan not shown",
                get_note(&["a"], 1, &[("a", vec![0])], &["a"])
            );
            assert_eq!(
                "Output limited to 65536 bytes: the last 3 plans and values of 1 change not shown",
                get_note(&["a"], 3, &[("a", vec![0])], &[])
            );
        }

        #[test]
        fn all_plans() {
            assert_eq!(
                "Output limited to 65536 bytes: the plan not shown",
                get_note(&[], 1, &[], &[])
            );
            assert_eq!(
                "Output limited to 65536 bytes: all 2 plans not shown",
                get_note(&[], 2, &[], &[])
            );
        }
    }
}
