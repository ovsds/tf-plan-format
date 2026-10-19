use crate::types;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum RawValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<RawValue>),
    Object(RawValueMap),
    Null,
}

pub type RawValueMap = std::collections::HashMap<String, RawValue>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "op", content = "path")]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(ValueMap),
    Null,
    Sensitive,
    SensitiveChanged,
    Unknown,
    SensitiveUnknown,
}

impl Value {
    #[must_use]
    pub fn from_raw(raw: &RawValue) -> Self {
        match raw {
            RawValue::String(value) => Value::String(value.clone()),
            RawValue::Integer(value) => Value::Integer(*value),
            RawValue::Float(value) => Value::Float(*value),
            RawValue::Boolean(value) => Value::Boolean(*value),
            RawValue::Array(values) => {
                let mut new_values = Vec::new();
                for value in values {
                    new_values.push(Value::from_raw(value));
                }
                Value::Array(new_values)
            }
            RawValue::Object(map) => {
                let mut new_map = ValueMap::new();
                for (key, value) in map {
                    new_map.insert(key.clone(), Value::from_raw(value));
                }
                Value::Object(new_map)
            }
            RawValue::Null => Value::Null,
        }
    }
}

pub type ValueMap = std::collections::HashMap<String, Value>;

fn value_map_from_raw(raw_map: &RawValueMap) -> ValueMap {
    let mut new_map = ValueMap::new();
    for (key, value) in raw_map {
        new_map.insert(key.clone(), Value::from_raw(value));
    }
    new_map
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum BoolValue {
    Boolean(bool),
    Array(Vec<BoolValue>),
    Object(std::collections::HashMap<String, BoolValue>),
    Null,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RawResourceChangeChangeAction {
    Create,
    Read,
    Update,
    Delete,
    #[serde(rename = "no-op")]
    NoOp,
}

impl FromStr for RawResourceChangeChangeAction {
    type Err = types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<RawResourceChangeChangeAction>(format!("\"{s}\"").as_str()) {
            Ok(action) => Ok(action),
            Err(e) => Err(types::Error::chain("Failed to parse action".to_string(), e)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Ord, PartialOrd, Eq)]
pub enum Action {
    Create,
    DeleteCreate,
    Read,
    Update,
    Delete,
    NoOp,
    Unknown,
}

impl Action {
    #[must_use]
    pub fn from_actions(actions: &[RawResourceChangeChangeAction]) -> Action {
        if actions.len() == 2
            && actions.contains(&RawResourceChangeChangeAction::Create)
            && actions.contains(&RawResourceChangeChangeAction::Delete)
        {
            return Action::DeleteCreate;
        }
        if actions.len() == 1 {
            return match actions[0] {
                RawResourceChangeChangeAction::Create => Action::Create,
                RawResourceChangeChangeAction::Read => Action::Read,
                RawResourceChangeChangeAction::Update => Action::Update,
                RawResourceChangeChangeAction::Delete => Action::Delete,
                RawResourceChangeChangeAction::NoOp => Action::NoOp,
            };
        }
        Action::Unknown
    }

    /// # Errors
    /// Returns an error if any of the actions cannot be parsed
    pub fn from_strings(actions: &Vec<String>) -> Result<Action, types::Error> {
        let mut parsed_actions = Vec::new();
        for action in actions {
            match RawResourceChangeChangeAction::from_str(action) {
                Ok(action) => parsed_actions.push(action),
                Err(_) => {
                    return Err(types::Error::default(format!(
                        "Failed to parse action({action})"
                    )))
                }
            }
        }
        Ok(Action::from_actions(&parsed_actions))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RawResourceChangeChange {
    pub actions: Vec<RawResourceChangeChangeAction>,
    pub before: Option<RawValueMap>,
    pub after: Option<RawValueMap>,
    pub after_unknown: Option<BoolValue>,
    pub before_sensitive: Option<BoolValue>,
    pub after_sensitive: Option<BoolValue>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RawResourceChange {
    pub address: String,
    // mode: String,
    // #[serde(rename = "type")]
    // type_: String,
    pub name: String,
    // provider_name: String,
    pub change: RawResourceChangeChange,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RawOutputChange {
    pub actions: Vec<RawResourceChangeChangeAction>,
    pub before: Option<RawValue>,
    pub after: Option<RawValue>,
    pub after_unknown: Option<BoolValue>,
    pub before_sensitive: Option<BoolValue>,
    pub after_sensitive: Option<BoolValue>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawPlan {
    // format_version: String,
    // terraform_version: String,
    // planned_values
    pub resource_changes: Option<Vec<RawResourceChange>>,
    pub output_changes: Option<std::collections::HashMap<String, RawOutputChange>>,
    // configuration
    // timestamp: String,
    // errored: bool,
}

impl FromStr for RawPlan {
    type Err = types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<RawPlan>(s) {
            Ok(plan) => Ok(plan),
            Err(e) => Err(types::Error::chain("Failed to parse plan".to_string(), e)),
        }
    }
}

impl RawPlan {
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: &str) -> Result<Self, types::Error> {
        let raw_file = std::fs::read_to_string(path)
            .map_err(|e| types::Error::chain(format!("Failed to read file({path})"), e))?;
        RawPlan::from_str(&raw_file)
            .map_err(|e| types::Error::chain(format!("Failed to parse file({path})"), e))
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Change {
    pub address: String,
    pub name: String,
    pub action: Action,
    pub before: Option<ValueMap>,
    pub after: Option<ValueMap>,
    pub raw: RawResourceChange,
}

fn get_child_flag(flag: &BoolValue, key: &str) -> BoolValue {
    match flag {
        BoolValue::Object(map) => match map.get(key) {
            Some(value) => value.clone(),
            None => BoolValue::Boolean(false),
        },
        _ => BoolValue::Boolean(is_flagged(flag)),
    }
}

fn is_flagged(flag: &BoolValue) -> bool {
    match flag {
        BoolValue::Boolean(value) => *value,
        _ => false,
    }
}

fn mask_sensitive(value: Value, sensitive: &BoolValue) -> Value {
    match (value.clone(), sensitive) {
        // A whole value flagged sensitive is masked whole: masking key by key would show the keys
        (Value::Object(value_map), _) if !is_flagged(sensitive) => {
            Value::Object(mask_sensitive_map(value_map, sensitive))
        }
        (Value::Array(value_array), BoolValue::Array(sensitive_array)) => {
            Value::Array(mask_sensitive_array(value_array, sensitive_array))
        }
        (Value::Unknown, _) if is_flagged(sensitive) => Value::SensitiveUnknown,
        _ if is_flagged(sensitive) => Value::Sensitive,
        _ => value,
    }
}

fn mask_sensitive_array(mut value_array: Vec<Value>, sensitive_array: &[BoolValue]) -> Vec<Value> {
    for (index, value) in value_array.iter_mut().enumerate() {
        let sensitive = sensitive_array
            .get(index)
            .unwrap_or(&BoolValue::Boolean(false));
        *value = mask_sensitive(value.clone(), sensitive);
    }
    value_array
}

fn mask_sensitive_map(mut value_map: ValueMap, sensitive: &BoolValue) -> ValueMap {
    for (key, value) in &mut value_map {
        *value = mask_sensitive(value.clone(), &get_child_flag(sensitive, key));
    }
    value_map
}

fn has_unknown(unknown: &BoolValue) -> bool {
    match unknown {
        BoolValue::Boolean(value) => *value,
        BoolValue::Array(array) => array.iter().any(has_unknown),
        BoolValue::Object(map) => map.values().any(has_unknown),
        BoolValue::Null => false,
    }
}

fn mark_unknown(value: Value, unknown: &BoolValue) -> Value {
    match (value, unknown) {
        (_, BoolValue::Boolean(true)) => Value::Unknown,
        (Value::Object(value_map), _) => Value::Object(mark_unknown_map(value_map, unknown)),
        (Value::Array(value_array), BoolValue::Array(unknown_array)) => {
            Value::Array(mark_unknown_array(value_array, unknown_array))
        }
        // Unknown parts of an absent or null value exist only in the flags
        (Value::Null, BoolValue::Object(_)) if has_unknown(unknown) => {
            Value::Object(mark_unknown_map(ValueMap::new(), unknown))
        }
        (Value::Null, BoolValue::Array(unknown_array)) if has_unknown(unknown) => {
            Value::Array(mark_unknown_array(Vec::new(), unknown_array))
        }
        (value, _) => value,
    }
}

fn mark_unknown_array(value_array: Vec<Value>, unknown_array: &[BoolValue]) -> Vec<Value> {
    let length = unknown_array
        .iter()
        .rposition(has_unknown)
        .map_or(0, |index| index + 1)
        .max(value_array.len());
    let mut values = value_array.into_iter();
    (0..length)
        .map(|index| {
            let value = values.next().unwrap_or(Value::Null);
            let unknown = unknown_array
                .get(index)
                .unwrap_or(&BoolValue::Boolean(false));
            mark_unknown(value, unknown)
        })
        .collect()
}

fn mark_unknown_map(mut value_map: ValueMap, unknown: &BoolValue) -> ValueMap {
    let BoolValue::Object(unknown_map) = unknown else {
        return value_map;
    };
    for (key, child_unknown) in unknown_map {
        let marked = match value_map.remove(key) {
            Some(value) => mark_unknown(value, child_unknown),
            None if has_unknown(child_unknown) => mark_unknown(Value::Null, child_unknown),
            None => continue,
        };
        value_map.insert(key.clone(), marked);
    }
    value_map
}

// Masked on both sides, a changed secret would compare equal to its old value and be hidden as unchanged
fn mark_sensitive_changes(
    before: &Value,
    after: Value,
    raw_before: &RawValue,
    raw_after: &RawValue,
) -> Value {
    match (before, after, raw_before, raw_after) {
        (Value::Sensitive, Value::Sensitive, raw_before, raw_after) if raw_before != raw_after => {
            Value::SensitiveChanged
        }
        (
            Value::Object(before),
            Value::Object(after),
            RawValue::Object(raw_before),
            RawValue::Object(raw_after),
        ) => Value::Object(mark_sensitive_changes_map(
            before, after, raw_before, raw_after,
        )),
        // Elements pair by index only while no insertion or removal shifted them
        (
            Value::Array(before),
            Value::Array(after),
            RawValue::Array(raw_before),
            RawValue::Array(raw_after),
        ) if raw_before.len() == raw_after.len() => Value::Array(
            after
                .into_iter()
                .enumerate()
                .map(|(index, after)| {
                    match (
                        before.get(index),
                        raw_before.get(index),
                        raw_after.get(index),
                    ) {
                        (Some(before), Some(raw_before), Some(raw_after)) => {
                            mark_sensitive_changes(before, after, raw_before, raw_after)
                        }
                        _ => after,
                    }
                })
                .collect(),
        ),
        (_, after, _, _) => after,
    }
}

fn mark_sensitive_changes_map(
    before: &ValueMap,
    mut after: ValueMap,
    raw_before: &RawValueMap,
    raw_after: &RawValueMap,
) -> ValueMap {
    for (key, value) in &mut after {
        if let (Some(before), Some(raw_before), Some(raw_after)) =
            (before.get(key), raw_before.get(key), raw_after.get(key))
        {
            *value = mark_sensitive_changes(
                before,
                std::mem::replace(value, Value::Null),
                raw_before,
                raw_after,
            );
        }
    }
    after
}

const NOT_FLAGGED: &BoolValue = &BoolValue::Boolean(false);

fn values_from_raw(
    raw_before: Option<&RawValueMap>,
    raw_after: Option<&RawValueMap>,
    before_sensitive: &BoolValue,
    after_sensitive: &BoolValue,
    after_unknown: &BoolValue,
) -> (Option<ValueMap>, Option<ValueMap>) {
    let before =
        raw_before.map(|raw| mask_sensitive_map(value_map_from_raw(raw), before_sensitive));
    let after = raw_after.map(|raw| {
        mask_sensitive_map(
            mark_unknown_map(value_map_from_raw(raw), after_unknown),
            after_sensitive,
        )
    });
    let after = match (&before, after, raw_before, raw_after) {
        (Some(before), Some(after), Some(raw_before), Some(raw_after)) => Some(
            mark_sensitive_changes_map(before, after, raw_before, raw_after),
        ),
        (_, after, _, _) => after,
    };
    (before, after)
}

impl Change {
    #[must_use]
    pub fn from_raw(raw: RawResourceChange) -> Self {
        let (before, after) = values_from_raw(
            raw.change.before.as_ref(),
            raw.change.after.as_ref(),
            raw.change.before_sensitive.as_ref().unwrap_or(NOT_FLAGGED),
            raw.change.after_sensitive.as_ref().unwrap_or(NOT_FLAGGED),
            raw.change.after_unknown.as_ref().unwrap_or(NOT_FLAGGED),
        );

        Change {
            address: raw.address.clone(),
            name: raw.name.clone(),
            action: Action::from_actions(&raw.change.actions),
            before,
            after,
            raw,
        }
    }
}

#[derive(Serialize, Debug, PartialEq, Default)]
pub struct Summary {
    pub add: usize,
    pub change: usize,
    pub destroy: usize,
}

impl Summary {
    #[must_use]
    pub fn from_changes(changes: &[Change]) -> Self {
        let mut summary = Summary::default();
        for change in changes {
            match change.action {
                Action::Create => summary.add += 1,
                Action::Delete => summary.destroy += 1,
                Action::DeleteCreate => {
                    summary.add += 1;
                    summary.destroy += 1;
                }
                Action::Update => summary.change += 1,
                Action::Read | Action::NoOp | Action::Unknown => {}
            }
        }
        summary
    }
}

#[derive(Serialize, Debug, PartialEq, Default)]
pub struct Outputs {
    pub before: ValueMap,
    pub after: ValueMap,
    pub actions: Vec<Action>,
}

impl Outputs {
    /// Treats the outputs as one resource whose attributes are the changed outputs
    #[must_use]
    pub fn from_raw(raw: &std::collections::HashMap<String, RawOutputChange>) -> Self {
        let mut actions = Vec::new();
        let mut raw_before = RawValueMap::new();
        let mut raw_after = RawValueMap::new();
        let mut before_sensitive = std::collections::HashMap::new();
        let mut after_sensitive = std::collections::HashMap::new();
        let mut after_unknown = std::collections::HashMap::new();
        for (name, change) in raw {
            let action = Action::from_actions(&change.actions);
            if action == Action::NoOp {
                continue;
            }
            actions.push(action);
            for (raw, map) in [
                (&change.before, &mut raw_before),
                (&change.after, &mut raw_after),
            ] {
                if let Some(value) = raw {
                    map.insert(name.clone(), value.clone());
                }
            }
            for (flag, map) in [
                (&change.before_sensitive, &mut before_sensitive),
                (&change.after_sensitive, &mut after_sensitive),
                (&change.after_unknown, &mut after_unknown),
            ] {
                if let Some(flag) = flag {
                    map.insert(name.clone(), flag.clone());
                }
            }
        }
        actions.sort();
        actions.dedup();

        let (before, after) = values_from_raw(
            Some(&raw_before),
            Some(&raw_after),
            &BoolValue::Object(before_sensitive),
            &BoolValue::Object(after_sensitive),
            &BoolValue::Object(after_unknown),
        );
        Outputs {
            before: before.unwrap_or_default(),
            after: after.unwrap_or_default(),
            actions,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Plan {
    title: String,
    pub changes: Vec<Change>,
    unique_actions: Vec<Action>,
    summary: Summary,
    pub outputs: Outputs,
    raw: RawPlan,
}

impl Plan {
    #[must_use]
    pub fn from_raw(raw: RawPlan) -> Self {
        let mut changes: Vec<Change> = Vec::new();
        if let Some(resource_changes) = &raw.resource_changes {
            for raw_change in resource_changes {
                changes.push(Change::from_raw(raw_change.clone()));
            }
        }
        let mut unique_actions: Vec<Action> = Vec::new();
        for change in &changes {
            if !unique_actions.contains(&change.action) {
                unique_actions.push(change.action.clone());
            }
        }
        unique_actions.sort();
        let summary = Summary::from_changes(&changes);
        let outputs = raw
            .output_changes
            .as_ref()
            .map(Outputs::from_raw)
            .unwrap_or_default();

        Plan {
            title: String::new(),
            changes,
            unique_actions,
            summary,
            outputs,
            raw,
        }
    }
}

fn get_path_components(path: &str) -> Vec<String> {
    std::path::Path::new(path)
        .components()
        .filter(|component| {
            matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::ParentDir
            )
        })
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

/// Shortens plan paths to what tells them apart: the file name when all plans share it, then the leading
/// directories all plans share, always keeping at least one component.
fn get_titles(paths: &[String]) -> Vec<String> {
    let mut components: Vec<Vec<String>> =
        paths.iter().map(|path| get_path_components(path)).collect();
    let shortest = components.iter().map(Vec::len).min().unwrap_or(0);
    if shortest > 1 && components.iter().map(|path| path.last()).all_equal() {
        for path in &mut components {
            path.pop();
        }
    }

    let shortest = components.iter().map(Vec::len).min().unwrap_or(0);
    let common = (0..shortest.saturating_sub(1))
        .take_while(|index| components.iter().map(|path| &path[*index]).all_equal())
        .count();
    components
        .iter()
        .map(|path| path[common..].join("/"))
        .collect()
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Data {
    pub plans: std::collections::HashMap<String, Plan>,
}

impl Data {
    #[must_use]
    pub fn new(mut plans: std::collections::HashMap<String, Plan>) -> Self {
        let paths: Vec<String> = plans.keys().cloned().collect();
        for (path, title) in paths.iter().zip(get_titles(&paths)) {
            if let Some(plan) = plans.get_mut(path) {
                plan.title = title;
            }
        }
        Data { plans }
    }

    /// # Errors
    /// Returns an error if any of the files cannot be read or parsed
    pub fn from_files(paths: &[String]) -> Result<Self, types::Error> {
        let mut plans: std::collections::HashMap<String, Plan> = std::collections::HashMap::new();
        for path_glob in paths {
            let glob = glob::glob(path_glob).map_err(|e| {
                types::Error::chain(format!("Failed to read file({path_glob}), invalid glob"), e)
            })?;

            let mut file_count = 0;
            for path in glob {
                let path_buf = path.map_err(|e| {
                    types::Error::chain(format!("Failed to read file({path_glob})"), e)
                })?;
                let Some(path) = path_buf.to_str() else {
                    return Err(types::Error::default(format!(
                        "Failed to read file({path_glob}), invalid path"
                    )));
                };
                let plan = RawPlan::from_file(path)
                    .map_err(|e| types::Error::chain(format!("Failed to read file({path})"), e))?;
                plans.insert(path.to_string(), Plan::from_raw(plan));

                file_count += 1;
            }

            if file_count == 0 {
                return Err(types::Error::default(format!(
                    "Failed to read file({path_glob}). No files found"
                )));
            }
        }
        Ok(Data::new(plans))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils;

    #[derive(Clone, Copy)]
    pub enum Tool {
        Terraform,
        OpenTofu,
    }

    #[derive(Clone, Copy)]
    pub enum PlanType {
        Create,
        Delete,
        DeleteCreate,
        NoOp,
        NoResources,
        Outputs,
        Read,
        Sensitive,
        Update,
        UpdateArray,
    }

    fn get_test_data_plans() -> Vec<(Tool, PlanType)> {
        let mut plans = Vec::new();
        for tool in [Tool::Terraform, Tool::OpenTofu] {
            for plan_type in [
                PlanType::Create,
                PlanType::Delete,
                PlanType::DeleteCreate,
                PlanType::NoOp,
                PlanType::NoResources,
                PlanType::Outputs,
                PlanType::Read,
                PlanType::Sensitive,
                PlanType::Update,
                PlanType::UpdateArray,
            ] {
                plans.push((tool, plan_type));
            }
        }
        return plans;
    }

    fn get_test_data_files() -> Vec<String> {
        let mut files = Vec::new();
        for (tool, plan_type) in get_test_data_plans() {
            files.push(get_test_plan_file(&tool, &plan_type));
        }
        return files;
    }

    pub fn get_test_data() -> Data {
        let mut plans: std::collections::HashMap<String, Plan> = std::collections::HashMap::new();

        for (tool, plan_type) in get_test_data_plans() {
            plans.insert(
                get_test_plan_file(&tool, &plan_type),
                get_test_plan(&tool, &plan_type),
            );
        }

        return Data::new(plans);
    }

    pub fn get_test_plan_file(tool: &Tool, plan_type: &PlanType) -> String {
        let file = match tool {
            Tool::Terraform => "terraform",
            Tool::OpenTofu => "opentofu",
        };
        let folder = match plan_type {
            PlanType::Create => "create",
            PlanType::Delete => "delete",
            PlanType::DeleteCreate => "delete-create",
            PlanType::NoOp => "no-op",
            PlanType::NoResources => "no-resources",
            PlanType::Outputs => "outputs",
            PlanType::Read => "read",
            PlanType::Sensitive => "sensitive",
            PlanType::Update => "update",
            PlanType::UpdateArray => "update-array",
        };
        return utils::test::get_test_data_file_path(&format!("plans/{folder}/{file}.tfplan.json"));
    }

    pub fn get_test_plan(tool: &Tool, plan_type: &PlanType) -> Plan {
        let json = get_test_plan_json(tool, plan_type);
        let raw = RawPlan::from_str(&json).unwrap();
        return Plan::from_raw(raw);
    }

    pub fn get_test_plan_json(tool: &Tool, plan_type: &PlanType) -> String {
        let file = get_test_plan_file(tool, plan_type);
        return std::fs::read_to_string(file).unwrap();
    }

    mod action {
        use super::*;

        mod from_actions {
            use super::*;
            #[test]
            fn create() {
                let actions = vec![RawResourceChangeChangeAction::Create];
                assert_eq!(Action::from_actions(&actions), Action::Create);
            }

            #[test]
            fn delete() {
                let actions = vec![RawResourceChangeChangeAction::Delete];
                assert_eq!(Action::from_actions(&actions), Action::Delete);
            }

            #[test]
            fn delete_create() {
                let actions = vec![
                    RawResourceChangeChangeAction::Create,
                    RawResourceChangeChangeAction::Delete,
                ];
                assert_eq!(Action::from_actions(&actions), Action::DeleteCreate);
            }

            #[test]
            fn no_op() {
                let actions = vec![RawResourceChangeChangeAction::NoOp];
                assert_eq!(Action::from_actions(&actions), Action::NoOp);
            }

            #[test]
            fn read() {
                let actions = vec![RawResourceChangeChangeAction::Read];
                assert_eq!(Action::from_actions(&actions), Action::Read);
            }

            #[test]
            fn update() {
                let actions = vec![RawResourceChangeChangeAction::Update];
                assert_eq!(Action::from_actions(&actions), Action::Update);
            }

            #[test]
            fn unknown() {
                let actions = vec![];
                assert_eq!(Action::from_actions(&actions), Action::Unknown);
            }
        }

        mod from_strings {

            use super::*;

            #[test]
            fn create() {
                assert_eq!(
                    Action::from_strings(&vec!["create".to_string()]).unwrap(),
                    Action::Create
                );
            }

            #[test]
            fn delete() {
                assert_eq!(
                    Action::from_strings(&vec!["delete".to_string()]).unwrap(),
                    Action::Delete
                );
            }

            #[test]
            fn delete_create() {
                assert_eq!(
                    Action::from_strings(&vec!["create".to_string(), "delete".to_string()])
                        .unwrap(),
                    Action::DeleteCreate
                );
            }

            #[test]
            fn no_op() {
                assert_eq!(
                    Action::from_strings(&vec!["no-op".to_string()]).unwrap(),
                    Action::NoOp
                );
            }

            #[test]
            fn read() {
                assert_eq!(
                    Action::from_strings(&vec!["read".to_string()]).unwrap(),
                    Action::Read
                );
            }

            #[test]
            fn update() {
                assert_eq!(
                    Action::from_strings(&vec!["update".to_string()]).unwrap(),
                    Action::Update
                );
            }

            #[test]
            fn unknown() {
                assert_eq!(Action::from_strings(&vec![]).unwrap(), Action::Unknown);
            }

            #[test]
            fn invalid() {
                let action = Action::from_strings(&vec!["invalid".to_string()]);
                assert_eq!(
                    action.unwrap_err().to_string(),
                    "Failed to parse action(invalid)"
                );
            }
        }
    }

    mod change {
        use super::*;

        mod from_raw {
            use super::*;

            fn get_raw(
                before: Option<RawValueMap>,
                after: Option<RawValueMap>,
                before_sensitive: Option<BoolValue>,
                after_sensitive: Option<BoolValue>,
            ) -> RawResourceChange {
                RawResourceChange {
                    address: "address".to_string(),
                    name: "name".to_string(),
                    change: RawResourceChangeChange {
                        actions: vec![RawResourceChangeChangeAction::Create],
                        before,
                        after,
                        after_unknown: None,
                        before_sensitive,
                        after_sensitive,
                    },
                }
            }

            #[test]
            fn full() {
                let path = utils::test::get_test_data_file_path("plans/artificial/full.json");
                let raw = RawPlan::from_file(&path).unwrap();

                let _ = Plan::from_raw(raw);
            }

            #[test]
            fn empty() {
                let raw = get_raw(None, None, None, None);
                let change = Change::from_raw(raw);

                assert_eq!(change.address, "address");
                assert_eq!(change.name, "name");
                assert_eq!(change.action, Action::Create);
                assert_eq!(change.before, None);
                assert_eq!(change.after, None);
            }

            #[test]
            fn sesitive_before_true() {
                let mut before = RawValueMap::new();
                before.insert("key".to_string(), RawValue::String("value".to_string()));

                let raw = get_raw(Some(before), None, Some(BoolValue::Boolean(true)), None);
                let change = Change::from_raw(raw);

                let mut expected_before = ValueMap::new();
                expected_before.insert("key".to_string(), Value::Sensitive);
                assert_eq!(change.before, Some(expected_before));
            }

            #[test]
            fn sesitive_after_true() {
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::String("value".to_string()));

                let raw = get_raw(None, Some(after), None, Some(BoolValue::Boolean(true)));
                let change = Change::from_raw(raw);

                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::Sensitive);
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn sesitive_before_false() {
                let mut before = RawValueMap::new();
                before.insert("key".to_string(), RawValue::String("value".to_string()));

                let raw = get_raw(Some(before), None, Some(BoolValue::Boolean(false)), None);
                let change = Change::from_raw(raw);
                let mut expected_before = ValueMap::new();
                expected_before.insert("key".to_string(), Value::String("value".to_string()));

                assert_eq!(change.before, Some(expected_before));
            }

            #[test]
            fn sesitive_after_false() {
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::String("value".to_string()));

                let raw = get_raw(None, Some(after), None, Some(BoolValue::Boolean(false)));
                let change = Change::from_raw(raw);
                let mut after = ValueMap::new();
                after.insert("key".to_string(), Value::String("value".to_string()));

                assert_eq!(change.after, Some(after));
            }

            #[test]
            fn sesitive_before_key_true() {
                let mut before = RawValueMap::new();
                before.insert("key".to_string(), RawValue::String("value".to_string()));

                let mut before_sensitive = std::collections::HashMap::new();
                before_sensitive.insert("key".to_string(), BoolValue::Boolean(true));

                let raw = get_raw(
                    Some(before),
                    None,
                    Some(BoolValue::Object(before_sensitive)),
                    None,
                );
                let change = Change::from_raw(raw);

                let mut expected_before = ValueMap::new();
                expected_before.insert("key".to_string(), Value::Sensitive);
                assert_eq!(change.before, Some(expected_before));
            }

            #[test]
            fn sesitive_after_key_true() {
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::String("value".to_string()));

                let mut after_sensitive = std::collections::HashMap::new();
                after_sensitive.insert("key".to_string(), BoolValue::Boolean(true));

                let raw = get_raw(
                    None,
                    Some(after.clone()),
                    None,
                    Some(BoolValue::Object(after_sensitive)),
                );
                let change = Change::from_raw(raw);

                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::Sensitive);
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn sensitive_after_array() {
                let mut after = RawValueMap::new();
                after.insert(
                    "key".to_string(),
                    RawValue::Array(vec![
                        RawValue::String("true".to_string()),
                        RawValue::String("false".to_string()),
                        RawValue::String("null".to_string()),
                        RawValue::String("absent".to_string()),
                    ]),
                );

                let mut after_sensitive = std::collections::HashMap::new();
                after_sensitive.insert(
                    "key".to_string(),
                    BoolValue::Array(vec![
                        BoolValue::Boolean(true),
                        BoolValue::Boolean(false),
                        BoolValue::Null,
                    ]),
                );

                let raw = get_raw(
                    None,
                    Some(after.clone()),
                    None,
                    Some(BoolValue::Object(after_sensitive)),
                );
                let change = Change::from_raw(raw);

                let mut expected_after = ValueMap::new();
                expected_after.insert(
                    "key".to_string(),
                    Value::Array(vec![
                        Value::Sensitive,
                        Value::String("false".to_string()),
                        Value::String("null".to_string()),
                        Value::String("absent".to_string()),
                    ]),
                );
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn sensitive_before_array() {
                let mut before = RawValueMap::new();
                before.insert(
                    "key".to_string(),
                    RawValue::Array(vec![
                        RawValue::String("true".to_string()),
                        RawValue::String("false".to_string()),
                        RawValue::String("null".to_string()),
                        RawValue::String("absent".to_string()),
                    ]),
                );

                let mut before_sensitive = std::collections::HashMap::new();
                before_sensitive.insert(
                    "key".to_string(),
                    BoolValue::Array(vec![
                        BoolValue::Boolean(true),
                        BoolValue::Boolean(false),
                        BoolValue::Null,
                    ]),
                );

                let raw = get_raw(
                    Some(before.clone()),
                    None,
                    Some(BoolValue::Object(before_sensitive)),
                    None,
                );
                let change = Change::from_raw(raw);

                let mut expected_before = ValueMap::new();
                expected_before.insert(
                    "key".to_string(),
                    Value::Array(vec![
                        Value::Sensitive,
                        Value::String("false".to_string()),
                        Value::String("null".to_string()),
                        Value::String("absent".to_string()),
                    ]),
                );
                assert_eq!(change.before, Some(expected_before));
            }

            fn get_raw_unknown(
                after: RawValueMap,
                after_unknown: BoolValue,
                after_sensitive: Option<BoolValue>,
            ) -> RawResourceChange {
                let mut raw = get_raw(None, Some(after), None, after_sensitive);
                raw.change.after_unknown = Some(after_unknown);
                raw
            }

            fn bool_object(entries: Vec<(&str, BoolValue)>) -> BoolValue {
                BoolValue::Object(
                    entries
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value))
                        .collect(),
                )
            }

            #[test]
            fn unknown_after_key_true() {
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::Null);
                after.insert("known".to_string(), RawValue::String("value".to_string()));

                let unknown = bool_object(vec![
                    ("key", BoolValue::Boolean(true)),
                    ("known", BoolValue::Boolean(false)),
                ]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, None));

                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::Unknown);
                expected_after.insert("known".to_string(), Value::String("value".to_string()));
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_absent_key() {
                let unknown = bool_object(vec![
                    ("absent_true", BoolValue::Boolean(true)),
                    ("absent_false", BoolValue::Boolean(false)),
                    ("absent_object", bool_object(vec![])),
                ]);
                let change = Change::from_raw(get_raw_unknown(RawValueMap::new(), unknown, None));

                let mut expected_after = ValueMap::new();
                expected_after.insert("absent_true".to_string(), Value::Unknown);
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_absent_nested() {
                let mut after = RawValueMap::new();
                after.insert("null_object".to_string(), RawValue::Null);
                let unknown = bool_object(vec![
                    (
                        "absent_array",
                        BoolValue::Array(vec![bool_object(vec![("id", BoolValue::Boolean(true))])]),
                    ),
                    (
                        "null_object",
                        bool_object(vec![("id", BoolValue::Boolean(true))]),
                    ),
                    (
                        "absent_known",
                        BoolValue::Array(vec![bool_object(vec![(
                            "id",
                            BoolValue::Boolean(false),
                        )])]),
                    ),
                ]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, None));

                let mut expected_id = ValueMap::new();
                expected_id.insert("id".to_string(), Value::Unknown);
                let mut expected_after = ValueMap::new();
                expected_after.insert(
                    "absent_array".to_string(),
                    Value::Array(vec![Value::Object(expected_id.clone())]),
                );
                expected_after.insert("null_object".to_string(), Value::Object(expected_id));
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_array_longer_flags() {
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::Array(vec![]));

                let unknown = bool_object(vec![(
                    "key",
                    BoolValue::Array(vec![BoolValue::Boolean(true), BoolValue::Boolean(false)]),
                )]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, None));

                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::Array(vec![Value::Unknown]));
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_map() {
                let mut inner = RawValueMap::new();
                inner.insert("known".to_string(), RawValue::String("value".to_string()));
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::Object(inner));

                let unknown = bool_object(vec![(
                    "key",
                    bool_object(vec![("absent", BoolValue::Boolean(true))]),
                )]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, None));

                let mut expected_inner = ValueMap::new();
                expected_inner.insert("known".to_string(), Value::String("value".to_string()));
                expected_inner.insert("absent".to_string(), Value::Unknown);
                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::Object(expected_inner));
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_whole_map() {
                let mut inner = RawValueMap::new();
                inner.insert("known".to_string(), RawValue::String("value".to_string()));
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::Object(inner));

                let unknown = bool_object(vec![("key", BoolValue::Boolean(true))]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, None));

                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::Unknown);
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_array() {
                let mut after = RawValueMap::new();
                after.insert(
                    "key".to_string(),
                    RawValue::Array(vec![
                        RawValue::Null,
                        RawValue::String("false".to_string()),
                        RawValue::String("null".to_string()),
                        RawValue::String("absent".to_string()),
                    ]),
                );

                let unknown = bool_object(vec![(
                    "key",
                    BoolValue::Array(vec![
                        BoolValue::Boolean(true),
                        BoolValue::Boolean(false),
                        BoolValue::Null,
                    ]),
                )]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, None));

                let mut expected_after = ValueMap::new();
                expected_after.insert(
                    "key".to_string(),
                    Value::Array(vec![
                        Value::Unknown,
                        Value::String("false".to_string()),
                        Value::String("null".to_string()),
                        Value::String("absent".to_string()),
                    ]),
                );
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_not_object() {
                let mut after = RawValueMap::new();
                after.insert("key".to_string(), RawValue::String("value".to_string()));

                let change = Change::from_raw(get_raw_unknown(
                    after.clone(),
                    BoolValue::Boolean(false),
                    None,
                ));

                let mut expected_after = ValueMap::new();
                expected_after.insert("key".to_string(), Value::String("value".to_string()));
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn unknown_after_sensitive() {
                let mut after = RawValueMap::new();
                after.insert("present".to_string(), RawValue::Null);

                let unknown = bool_object(vec![
                    ("present", BoolValue::Boolean(true)),
                    ("absent", BoolValue::Boolean(true)),
                    ("not_sensitive", BoolValue::Boolean(true)),
                ]);
                let sensitive = bool_object(vec![
                    ("present", BoolValue::Boolean(true)),
                    ("absent", BoolValue::Boolean(true)),
                ]);
                let change = Change::from_raw(get_raw_unknown(after, unknown, Some(sensitive)));

                let mut expected_after = ValueMap::new();
                expected_after.insert("present".to_string(), Value::SensitiveUnknown);
                expected_after.insert("absent".to_string(), Value::SensitiveUnknown);
                expected_after.insert("not_sensitive".to_string(), Value::Unknown);
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn sensitive_whole_object() {
                let mut inner = RawValueMap::new();
                inner.insert("alice".to_string(), RawValue::String("admin".to_string()));
                let mut after = RawValueMap::new();
                after.insert("users".to_string(), RawValue::Object(inner));

                let sensitive = bool_object(vec![("users", BoolValue::Boolean(true))]);
                let change = Change::from_raw(get_raw(None, Some(after), None, Some(sensitive)));

                let mut expected_after = ValueMap::new();
                expected_after.insert("users".to_string(), Value::Sensitive);
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn sensitive_array_insert_not_changed() {
                let mut before = RawValueMap::new();
                before.insert(
                    "keys".to_string(),
                    RawValue::Array(vec![
                        RawValue::String("s1".to_string()),
                        RawValue::String("s2".to_string()),
                    ]),
                );
                let mut after = RawValueMap::new();
                after.insert(
                    "keys".to_string(),
                    RawValue::Array(vec![
                        RawValue::String("s0".to_string()),
                        RawValue::String("s1".to_string()),
                        RawValue::String("s2".to_string()),
                    ]),
                );

                let before_sensitive = bool_object(vec![(
                    "keys",
                    BoolValue::Array(vec![BoolValue::Boolean(true); 2]),
                )]);
                let after_sensitive = bool_object(vec![(
                    "keys",
                    BoolValue::Array(vec![BoolValue::Boolean(true); 3]),
                )]);
                let change = Change::from_raw(get_raw(
                    Some(before),
                    Some(after),
                    Some(before_sensitive),
                    Some(after_sensitive),
                ));

                let mut expected_after = ValueMap::new();
                expected_after.insert("keys".to_string(), Value::Array(vec![Value::Sensitive; 3]));
                assert_eq!(change.after, Some(expected_after));
            }

            #[test]
            fn sensitive_changed() {
                let mut before = RawValueMap::new();
                before.insert("changed".to_string(), RawValue::String("a".to_string()));
                before.insert("same".to_string(), RawValue::String("a".to_string()));
                before.insert(
                    "nested".to_string(),
                    RawValue::Array(vec![RawValue::String("a".to_string())]),
                );
                let mut after = before.clone();
                after.insert("changed".to_string(), RawValue::String("b".to_string()));
                after.insert(
                    "nested".to_string(),
                    RawValue::Array(vec![RawValue::String("b".to_string())]),
                );

                let sensitive = bool_object(vec![
                    ("changed", BoolValue::Boolean(true)),
                    ("same", BoolValue::Boolean(true)),
                    ("nested", BoolValue::Array(vec![BoolValue::Boolean(true)])),
                ]);
                let change = Change::from_raw(get_raw(
                    Some(before),
                    Some(after),
                    Some(sensitive.clone()),
                    Some(sensitive),
                ));

                let mut expected_before = ValueMap::new();
                expected_before.insert("changed".to_string(), Value::Sensitive);
                expected_before.insert("same".to_string(), Value::Sensitive);
                expected_before.insert("nested".to_string(), Value::Array(vec![Value::Sensitive]));
                assert_eq!(change.before, Some(expected_before));

                let mut expected_after = ValueMap::new();
                expected_after.insert("changed".to_string(), Value::SensitiveChanged);
                expected_after.insert("same".to_string(), Value::Sensitive);
                expected_after.insert(
                    "nested".to_string(),
                    Value::Array(vec![Value::SensitiveChanged]),
                );
                assert_eq!(change.after, Some(expected_after));
            }
        }
    }

    mod summary {
        use super::*;

        fn get_change(actions: Vec<RawResourceChangeChangeAction>) -> Change {
            Change::from_raw(RawResourceChange {
                address: "address".to_string(),
                name: "name".to_string(),
                change: RawResourceChangeChange {
                    actions,
                    before: None,
                    after: None,
                    after_unknown: None,
                    before_sensitive: None,
                    after_sensitive: None,
                },
            })
        }

        #[test]
        fn from_changes() {
            let changes = vec![
                get_change(vec![RawResourceChangeChangeAction::Create]),
                get_change(vec![RawResourceChangeChangeAction::Delete]),
                get_change(vec![
                    RawResourceChangeChangeAction::Delete,
                    RawResourceChangeChangeAction::Create,
                ]),
                get_change(vec![RawResourceChangeChangeAction::Update]),
                get_change(vec![RawResourceChangeChangeAction::Update]),
                get_change(vec![RawResourceChangeChangeAction::Read]),
                get_change(vec![RawResourceChangeChangeAction::NoOp]),
                get_change(vec![]),
            ];

            assert_eq!(
                Summary::from_changes(&changes),
                Summary {
                    add: 2,
                    change: 2,
                    destroy: 2,
                }
            );
        }

        #[test]
        fn empty() {
            assert_eq!(Summary::from_changes(&[]), Summary::default());
        }
    }

    mod outputs {
        use super::*;

        fn get_raw(
            actions: Vec<RawResourceChangeChangeAction>,
            before: Option<RawValue>,
            after: Option<RawValue>,
        ) -> RawOutputChange {
            RawOutputChange {
                actions,
                before,
                after,
                after_unknown: Some(BoolValue::Boolean(false)),
                before_sensitive: Some(BoolValue::Boolean(false)),
                after_sensitive: Some(BoolValue::Boolean(false)),
            }
        }

        fn from_raw(changes: Vec<(&str, RawOutputChange)>) -> Outputs {
            let changes = changes
                .into_iter()
                .map(|(name, change)| (name.to_string(), change))
                .collect();
            Outputs::from_raw(&changes)
        }

        fn value_map(entries: Vec<(&str, Value)>) -> ValueMap {
            entries
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect()
        }

        fn string(value: &str) -> RawValue {
            RawValue::String(value.to_string())
        }

        #[test]
        fn create() {
            let outputs = from_raw(vec![(
                "name",
                get_raw(
                    vec![RawResourceChangeChangeAction::Create],
                    None,
                    Some(string("after")),
                ),
            )]);

            assert_eq!(outputs.before, ValueMap::new());
            assert_eq!(
                outputs.after,
                value_map(vec![("name", Value::String("after".to_string()))])
            );
            assert_eq!(outputs.actions, vec![Action::Create]);
        }

        #[test]
        fn update() {
            let mut object = RawValueMap::new();
            object.insert(
                "key".to_string(),
                RawValue::Array(vec![RawValue::Integer(1)]),
            );
            let outputs = from_raw(vec![(
                "name",
                get_raw(
                    vec![RawResourceChangeChangeAction::Update],
                    Some(string("before")),
                    Some(RawValue::Object(object)),
                ),
            )]);

            assert_eq!(
                outputs.before,
                value_map(vec![("name", Value::String("before".to_string()))])
            );
            assert_eq!(
                outputs.after,
                value_map(vec![(
                    "name",
                    Value::Object(value_map(vec![(
                        "key",
                        Value::Array(vec![Value::Integer(1)])
                    )]))
                )])
            );
            assert_eq!(outputs.actions, vec![Action::Update]);
        }

        #[test]
        fn delete() {
            let outputs = from_raw(vec![(
                "name",
                get_raw(
                    vec![RawResourceChangeChangeAction::Delete],
                    Some(string("before")),
                    None,
                ),
            )]);

            assert_eq!(
                outputs.before,
                value_map(vec![("name", Value::String("before".to_string()))])
            );
            assert_eq!(outputs.after, ValueMap::new());
            assert_eq!(outputs.actions, vec![Action::Delete]);
        }

        #[test]
        fn no_op() {
            let outputs = from_raw(vec![(
                "name",
                get_raw(
                    vec![RawResourceChangeChangeAction::NoOp],
                    Some(string("same")),
                    Some(string("same")),
                ),
            )]);

            assert_eq!(outputs, Outputs::default());
        }

        #[test]
        fn sensitive() {
            let mut change = get_raw(
                vec![RawResourceChangeChangeAction::Update],
                Some(string("before")),
                Some(RawValue::Array(vec![string("public"), string("secret")])),
            );
            change.before_sensitive = Some(BoolValue::Boolean(true));
            change.after_sensitive = Some(BoolValue::Array(vec![
                BoolValue::Boolean(false),
                BoolValue::Boolean(true),
            ]));
            let outputs = from_raw(vec![("name", change)]);

            assert_eq!(outputs.before, value_map(vec![("name", Value::Sensitive)]));
            assert_eq!(
                outputs.after,
                value_map(vec![(
                    "name",
                    Value::Array(vec![Value::String("public".to_string()), Value::Sensitive])
                )])
            );
        }

        #[test]
        fn unknown() {
            let mut change = get_raw(
                vec![RawResourceChangeChangeAction::Update],
                Some(string("before")),
                None,
            );
            change.after_unknown = Some(BoolValue::Boolean(true));
            let mut sensitive_change =
                get_raw(vec![RawResourceChangeChangeAction::Create], None, None);
            sensitive_change.after_unknown = Some(BoolValue::Boolean(true));
            sensitive_change.after_sensitive = Some(BoolValue::Boolean(true));
            let outputs = from_raw(vec![("name", change), ("sensitive", sensitive_change)]);

            assert_eq!(
                outputs.after,
                value_map(vec![
                    ("name", Value::Unknown),
                    ("sensitive", Value::SensitiveUnknown)
                ])
            );
            assert_eq!(outputs.actions, vec![Action::Create, Action::Update]);
        }

        #[test]
        fn absent_flags() {
            let outputs = from_raw(vec![(
                "name",
                RawOutputChange {
                    actions: vec![RawResourceChangeChangeAction::Create],
                    before: None,
                    after: Some(string("after")),
                    after_unknown: None,
                    before_sensitive: None,
                    after_sensitive: None,
                },
            )]);

            assert_eq!(
                outputs.after,
                value_map(vec![("name", Value::String("after".to_string()))])
            );
        }

        #[test]
        fn unique_actions() {
            let outputs = from_raw(vec![
                (
                    "a",
                    get_raw(
                        vec![RawResourceChangeChangeAction::Delete],
                        Some(string("a")),
                        None,
                    ),
                ),
                (
                    "b",
                    get_raw(
                        vec![RawResourceChangeChangeAction::Update],
                        Some(string("b")),
                        Some(string("c")),
                    ),
                ),
                (
                    "c",
                    get_raw(
                        vec![RawResourceChangeChangeAction::Delete],
                        Some(string("c")),
                        None,
                    ),
                ),
            ]);

            assert_eq!(outputs.actions, vec![Action::Update, Action::Delete]);
        }

        #[test]
        fn plan() {
            let plan = get_test_plan(&Tool::Terraform, &PlanType::Outputs);

            assert_eq!(
                plan.outputs.actions,
                vec![Action::Create, Action::Update, Action::Delete]
            );
            assert_eq!(
                plan.outputs.before,
                value_map(vec![
                    ("changed", Value::String("before".to_string())),
                    ("removed", Value::String("removed".to_string())),
                    ("secret", Value::Sensitive),
                    ("unknown", Value::String("foo".to_string())),
                ])
            );
            assert_eq!(
                plan.outputs.after,
                value_map(vec![
                    ("added", Value::String("added".to_string())),
                    ("changed", Value::String("after".to_string())),
                    ("secret", Value::SensitiveChanged),
                    ("unknown", Value::Unknown),
                ])
            );
        }
    }

    mod titles {
        use super::*;

        fn titles(paths: &[&str]) -> Vec<String> {
            let paths: Vec<String> = paths.iter().map(ToString::to_string).collect();
            get_titles(&paths)
        }

        #[test]
        fn same_file_name() {
            assert_eq!(
                titles(&[
                    "plans/github/ovsds/plan.json",
                    "plans/cloudflare/main/plan.json"
                ]),
                vec!["github/ovsds", "cloudflare/main"]
            );
        }

        #[test]
        fn different_file_names() {
            assert_eq!(
                titles(&[
                    "tests/data/plans/create/opentofu.tfplan.json",
                    "tests/data/plans/create/terraform.tfplan.json",
                    "tests/data/plans/update/opentofu.tfplan.json",
                ]),
                vec![
                    "create/opentofu.tfplan.json",
                    "create/terraform.tfplan.json",
                    "update/opentofu.tfplan.json",
                ]
            );
        }

        #[test]
        fn single() {
            assert_eq!(titles(&["plans/github/ovsds/plan.json"]), vec!["ovsds"]);
        }

        #[test]
        fn single_file_name_only() {
            assert_eq!(titles(&["plan.json"]), vec!["plan.json"]);
        }

        #[test]
        fn same_directory() {
            assert_eq!(
                titles(&["plans/terraform.json", "plans/opentofu.json"]),
                vec!["terraform.json", "opentofu.json"]
            );
        }

        #[test]
        fn nested() {
            assert_eq!(
                titles(&["plans/a/plan.json", "plans/a/b/plan.json"]),
                vec!["a", "a/b"]
            );
        }

        #[test]
        fn root_and_current_dir() {
            assert_eq!(
                titles(&["/plans/a/plan.json", "./plans/b/plan.json"]),
                vec!["a", "b"]
            );
        }

        #[test]
        fn empty() {
            assert_eq!(titles(&[]), Vec::<String>::new());
        }

        #[test]
        fn from_files() {
            let files = vec!["tests/data/plans/*/*.tfplan.json".to_string()];
            let data = Data::from_files(&files).unwrap();
            let file = get_test_plan_file(&Tool::OpenTofu, &PlanType::Create);
            assert_eq!(data.plans[&file].title, "create/opentofu.tfplan.json");
        }
    }

    mod value {
        use super::*;

        #[test]
        fn serialize_unknown() {
            assert_eq!(
                serde_json::to_string(&Value::Unknown).unwrap(),
                r#"{"op":"Unknown"}"#
            );
            assert_eq!(
                serde_json::to_string(&Value::SensitiveUnknown).unwrap(),
                r#"{"op":"SensitiveUnknown"}"#
            );
        }
    }

    mod raw_plan {
        use super::*;

        mod from_str {
            use super::*;

            macro_rules!tests {
                ($($name:ident, $tool:expr, $plan_type:expr)*) => {
                    $(
                        #[test]
                        fn $name() {
                            get_test_plan(&$tool, &$plan_type);
                        }
                    )*
                };
            }

            tests! {
                terraform_create, Tool::Terraform, PlanType::Create
                terraform_delete, Tool::Terraform, PlanType::Delete
                terraform_delete_create, Tool::Terraform, PlanType::DeleteCreate
                terraform_no_op, Tool::Terraform, PlanType::NoOp
                terraform_no_resources, Tool::Terraform, PlanType::NoResources
                terraform_outputs, Tool::Terraform, PlanType::Outputs
                terraform_read, Tool::Terraform, PlanType::Read
                terraform_sensitive, Tool::Terraform, PlanType::Sensitive
                terraform_update, Tool::Terraform, PlanType::Update
                terraform_update_array, Tool::Terraform, PlanType::UpdateArray
                opentofu_create, Tool::OpenTofu, PlanType::Create
                opentofu_delete, Tool::OpenTofu, PlanType::Delete
                opentofu_delete_create, Tool::OpenTofu, PlanType::DeleteCreate
                opentofu_no_op, Tool::OpenTofu, PlanType::NoOp
                opentofu_no_resources, Tool::OpenTofu, PlanType::NoResources
                opentofu_outputs, Tool::OpenTofu, PlanType::Outputs
                opentofu_read, Tool::OpenTofu, PlanType::Read
                opentofu_sensitive, Tool::OpenTofu, PlanType::Sensitive
                opentofu_update, Tool::OpenTofu, PlanType::Update
                opentofu_update_array, Tool::OpenTofu, PlanType::UpdateArray
            }

            #[test]
            fn invalid_json() {
                let plan = RawPlan::from_str("invalid json");
                assert_eq!(
                    plan.unwrap_err().full_message(),
                    "Failed to parse plan. expected value at line 1 column 1"
                );
            }
        }

        mod from_file {
            use super::*;

            #[test]
            fn full() {
                let path = utils::test::get_test_data_file_path("plans/artificial/full.json");
                RawPlan::from_file(&path).unwrap();
            }

            #[test]
            fn no_resource_changes() {
                let path = utils::test::get_test_data_file_path(
                    "plans/artificial/no-resource-changes.json",
                );
                RawPlan::from_file(&path).unwrap();
            }

            #[test]
            fn invalid_path() {
                let plan = RawPlan::from_file("invalid path");
                assert_eq!(
                    plan.unwrap_err().full_message(),
                    "Failed to read file(invalid path). No such file or directory (os error 2)"
                );
            }

            #[test]
            fn invalid_json() {
                let path = utils::test::get_test_data_file_path("plans/artificial/invalid.json");
                let plan = RawPlan::from_file(&path);
                assert_eq!(
                    plan.unwrap_err().full_message(),
                    "Failed to parse file(tests/data/plans/artificial/invalid.json). Failed to parse plan. invalid type: string \"invalid\", expected a sequence at line 2 column 31"
                );
            }
        }
    }

    mod data {
        use super::*;

        mod from_files {
            use super::*;

            #[test]
            fn default() {
                let files = get_test_data_files();
                let data = Data::from_files(&files).unwrap();
                assert_eq!(data, get_test_data());
            }

            #[test]
            fn glob() {
                let files = vec!["tests/data/plans/*/*.tfplan.json".to_string()];
                let data = Data::from_files(&files).unwrap();
                assert_eq!(data, get_test_data());
            }

            #[test]
            fn invalid_glob() {
                let files = vec!["*****".to_string()];
                let data = Data::from_files(&files);
                assert_eq!(
                    data.unwrap_err().full_message(),
                    "Failed to read file(*****), invalid glob. Pattern syntax error near position 2: wildcards are either regular `*` or recursive `**`"
                );
            }

            #[test]
            fn no_files() {
                let data = Data::from_files(&["invalid path".to_string()]);
                assert_eq!(
                    data.unwrap_err().to_string(),
                    "Failed to read file(invalid path). No files found"
                );
            }
        }
    }
}
