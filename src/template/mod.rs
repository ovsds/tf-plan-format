use crate::tf;
use crate::types;
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

    let blocks = get_values_blocks(data, &hidden_actions, show_changed_values);
    let mut limits = Limits::new(max_length, data.plans.len());
    result = collapse_until_fits(&mut limits, &blocks, result, |limits| render(Some(limits)))?;
    if result.len() > max_length {
        limits.plan_count = get_fitting_plan_count(limits.plan_count, |plan_count| {
            limits.plan_count = plan_count;
            Ok(render(Some(&limits))?.len() <= max_length)
        })?;
        // Collapsing everything was needed only before plans were dropped: the kept plans may fit with less
        let plan_count = limits.plan_count;
        limits = Limits::new(max_length, data.plans.len());
        limits.plan_count = plan_count;
        let kept_blocks: Vec<&ValuesBlock> = blocks
            .iter()
            .filter(|block| block.plan < plan_count)
            .collect();
        result = render(Some(&limits))?;
        result = collapse_until_fits(&mut limits, &kept_blocks, result, |limits| {
            render(Some(limits))
        })?;
    }
    Ok(result)
}

/// Collapses blocks largest first until the output fits or every block is collapsed
fn collapse_until_fits<B: std::borrow::Borrow<ValuesBlock>>(
    limits: &mut Limits,
    blocks: &[B],
    mut result: String,
    render: impl Fn(&Limits) -> Result<String, types::Error>,
) -> Result<String, types::Error> {
    let mut blocks = blocks.iter().map(std::borrow::Borrow::borrow);
    while result.len() > limits.max_length && blocks.len() > 0 {
        // The savings estimate the re-render, which also accounts for the note
        let excess = result.len() - limits.max_length;
        let mut saving = 0;
        for block in blocks.by_ref() {
            saving += block.saving;
            limits.collapse(block);
            if saving >= excess {
                break;
            }
        }
        result = render(limits)?;
    }
    Ok(result)
}

/// The most plans that fit when the last ones are dropped, knowing that all of them do not; none if even one
/// plan does not fit
fn get_fitting_plan_count(
    plan_count: usize,
    mut fits: impl FnMut(usize) -> Result<bool, types::Error>,
) -> Result<usize, types::Error> {
    let (mut low, mut high) = (0, plan_count.saturating_sub(1));
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if fits(middle)? {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    Ok(low)
}

enum ValuesBlockKind {
    Change(usize),
    Outputs,
}

struct ValuesBlock {
    plan: usize,
    kind: ValuesBlockKind,
    saving: usize,
}

/// Values blocks the github template renders that collapsing shortens, largest saving first, ties in render
/// order
fn get_values_blocks(
    data: &tf::Data,
    hidden_actions: &[tf::Action],
    show_changed_values: bool,
) -> Vec<ValuesBlock> {
    let mut blocks = Vec::new();
    for (plan_index, plan) in data.plans.values().enumerate() {
        let changes = plan
            .changes
            .iter()
            .filter(|change| tera::keeps_action(&change.action, hidden_actions, false))
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
        blocks.extend(changes.chain(outputs).filter_map(|(kind, before, after)| {
            let values = tera::render_values(before, after, show_changed_values);
            let saving = (values.len() + "```\n\n```".len())
                .checked_sub(tera::COLLAPSED_VALUES.len())
                .filter(|&saving| saving > 0)?;
            Some(ValuesBlock {
                plan: plan_index,
                kind,
                saving,
            })
        }));
    }
    blocks.sort_by_key(|block| std::cmp::Reverse(block.saving));
    blocks
}

/// Plans are identified by their position in render order, dropping only ever removes the last ones
struct Limits {
    max_length: usize,
    plan_count: usize,
    collapsed_changes: Vec<Vec<usize>>,
    collapsed_outputs: Vec<usize>,
}

impl Limits {
    fn new(max_length: usize, plan_count: usize) -> Self {
        Limits {
            max_length,
            plan_count,
            collapsed_changes: vec![Vec::new(); plan_count],
            collapsed_outputs: Vec::new(),
        }
    }

    fn collapse(&mut self, block: &ValuesBlock) {
        match block.kind {
            ValuesBlockKind::Change(index) => self.collapsed_changes[block.plan].push(index),
            ValuesBlockKind::Outputs => self.collapsed_outputs.push(block.plan),
        }
    }

    fn note(&self) -> String {
        let mut parts = Vec::new();
        match (
            self.plan_count,
            self.collapsed_changes.len() - self.plan_count,
        ) {
            (_, 0) => {}
            (0, 1) => parts.push("the plan".to_string()),
            (0, dropped) => parts.push(format!("all {dropped} plans")),
            (_, 1) => parts.push("the last plan".to_string()),
            (_, dropped) => parts.push(format!("the last {dropped} plans")),
        }
        let collapsed_changes = self.collapsed_changes[..self.plan_count]
            .iter()
            .map(Vec::len)
            .sum();
        if collapsed_changes > 0 {
            parts.push(format!("values of {}", count(collapsed_changes, "change")));
        }
        let collapsed_outputs = self
            .collapsed_outputs
            .iter()
            .filter(|&&plan| plan < self.plan_count)
            .count();
        if collapsed_outputs > 0 {
            parts.push(format!("outputs of {}", count(collapsed_outputs, "plan")));
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
            "plan_count".to_string(),
            tera::RenderOptionValue::Count(self.plan_count),
        );
        options.insert(
            "collapsed_changes".to_string(),
            tera::RenderOptionValue::IndexLists(self.collapsed_changes.clone()),
        );
        options.insert(
            "collapsed_outputs".to_string(),
            tera::RenderOptionValue::Indices(self.collapsed_outputs.clone()),
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

        const COLLAPSED: &str = tera::COLLAPSED_VALUES;

        fn get_expected_default() -> String {
            utils::test::get_test_data_file_contents("tera/renders/github_markdown/default.md")
        }

        fn render_capped(data: &tf::Data, max_length: usize) -> String {
            render_github(data, false, false, false, Some(max_length)).unwrap()
        }

        fn get_outputs_data() -> tf::Data {
            let tool = tf::tests::Tool::Terraform;
            let plan_type = tf::tests::PlanType::Outputs;
            tf::Data::new(std::collections::BTreeMap::from([(
                tf::tests::get_test_plan_file(&tool, &plan_type),
                tf::tests::get_test_plan(&tool, &plan_type),
            )]))
        }

        fn get_plan(plan_type: tf::tests::PlanType) -> tf::Plan {
            tf::tests::get_test_plan(&tf::tests::Tool::Terraform, &plan_type)
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
            // The kept plans show again the values that fit once the last plans are dropped
            assert!(result.contains("```"));
            assert!(result.contains("no-resources/terraform.tfplan.json - "));
            assert!(!result.contains("outputs/opentofu.tfplan.json - "));
            assert_eq!(
                "_Output limited to 2000 bytes: the last 10 plans and values of 4 changes not shown_",
                get_note(&result)
            );
        }

        #[test]
        fn plan_key_with_special_characters() {
            let data = tf::Data::new(std::collections::BTreeMap::from([(
                "envs/[\"prod\"]/plan.json".to_string(),
                get_plan(tf::tests::PlanType::UpdateArray),
            )]));
            let full = render_github(&data, false, false, false, None).unwrap();
            let result = render_capped(&data, full.len() - 1);

            assert!(result.contains(&format!(
                "<summary>🔄terraform_data.foo-bar\n</summary>\n\n{COLLAPSED}\n\n</details>"
            )));
            assert_eq!(1, result.matches(COLLAPSED).count());
            assert_eq!(
                format!(
                    "_Output limited to {} bytes: values of 1 change not shown_",
                    full.len() - 1
                ),
                get_note(&result)
            );
        }

        #[test]
        fn never_collapses_small_blocks() {
            let mut small = get_plan(tf::tests::PlanType::Create);
            let mut change = get_plan(tf::tests::PlanType::Create).changes.remove(0);
            change.address = "terraform_data.small".to_string();
            change.after = Some(tf::ValueMap::from([(
                "a".to_string(),
                tf::Value::String("b".to_string()),
            )]));
            small.changes.push(change);
            let data = tf::Data::new(std::collections::BTreeMap::from([
                ("a/plan.json".to_string(), small),
                (
                    "b/plan.json".to_string(),
                    get_plan(tf::tests::PlanType::UpdateArray),
                ),
            ]));
            assert_eq!(2, get_values_blocks(&data, &[], false).len());

            let full = render_github(&data, false, false, false, None).unwrap();
            let mut dropped_with_small_shown = 0;
            for max_length in (0..full.len()).step_by(10) {
                let result = render_capped(&data, max_length);
                let small_shown = result
                    .contains("<summary>✅terraform_data.small\n</summary>\n\n```\na: \"b\"\n```");
                assert!(
                    small_shown || !result.contains("terraform_data.small"),
                    "{max_length}: {result}"
                );
                if small_shown && result.contains("the last plan") {
                    dropped_with_small_shown += 1;
                }
            }
            // A plan is dropped only once collapsing every candidate did not fit, a small candidate would be collapsed
            assert!(dropped_with_small_shown > 0);
        }

        #[test]
        fn drops_fewest_plans() {
            for max_length in 0..1100 {
                let mut renders = 0;
                let fits = |plan_count: usize| plan_count * 100 + 10 <= max_length;
                let plan_count = get_fitting_plan_count(10, |plan_count| {
                    renders += 1;
                    Ok(fits(plan_count))
                })
                .unwrap();

                assert!(fits(plan_count) || plan_count == 0, "{max_length}");
                assert!(plan_count == 9 || !fits(plan_count + 1), "{max_length}");
                assert!(renders <= 4, "{max_length}");
            }
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
            plans: usize,
            plan_count: usize,
            collapsed_changes: &[(usize, Vec<usize>)],
            collapsed_outputs: &[usize],
        ) -> String {
            let mut limits = Limits::new(65536, plans);
            limits.plan_count = plan_count;
            for (plan, indices) in collapsed_changes {
                limits.collapsed_changes[*plan].clone_from(indices);
            }
            limits.collapsed_outputs = collapsed_outputs.to_vec();
            limits.note()
        }

        #[test]
        fn changes() {
            assert_eq!(
                "Output limited to 65536 bytes: values of 1 change not shown",
                get_note(1, 1, &[(0, vec![0])], &[])
            );
            assert_eq!(
                "Output limited to 65536 bytes: values of 3 changes not shown",
                get_note(2, 2, &[(0, vec![0, 2]), (1, vec![1])], &[])
            );
        }

        #[test]
        fn outputs() {
            assert_eq!(
                "Output limited to 65536 bytes: outputs of 2 plans not shown",
                get_note(2, 2, &[], &[0, 1])
            );
        }

        #[test]
        fn changes_and_outputs() {
            assert_eq!(
                "Output limited to 65536 bytes: values of 2 changes and outputs of 1 plan not shown",
                get_note(1, 1, &[(0, vec![0, 1])], &[0])
            );
        }

        #[test]
        fn plans() {
            assert_eq!(
                "Output limited to 65536 bytes: the last plan, values of 1 change and outputs of 1 plan not shown",
                get_note(2, 1, &[(0, vec![0])], &[0])
            );
            assert_eq!(
                "Output limited to 65536 bytes: the last 3 plans and values of 1 change not shown",
                get_note(4, 1, &[(0, vec![0])], &[])
            );
        }

        #[test]
        fn dropped_plans_not_counted() {
            assert_eq!(
                "Output limited to 65536 bytes: the last plan and values of 1 change not shown",
                get_note(2, 1, &[(0, vec![0]), (1, vec![0, 1])], &[1])
            );
        }

        #[test]
        fn all_plans() {
            assert_eq!(
                "Output limited to 65536 bytes: the plan not shown",
                get_note(1, 0, &[(0, vec![0])], &[0])
            );
            assert_eq!(
                "Output limited to 65536 bytes: all 2 plans not shown",
                get_note(2, 0, &[], &[])
            );
        }
    }
}
