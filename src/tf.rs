use crate::types;
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
        };
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
    // after_unknown
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawPlan {
    // format_version: String,
    // terraform_version: String,
    // planned_values
    pub resource_changes: Option<Vec<RawResourceChange>>,
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

fn get_child_sensitive(sensitive: &BoolValue, key: &str) -> BoolValue {
    match sensitive {
        BoolValue::Object(map) => match map.get(key) {
            Some(value) => value.clone(),
            None => BoolValue::Boolean(false),
        },
        _ => BoolValue::Boolean(is_sensitive(sensitive)),
    }
}

fn is_sensitive(sensitive: &BoolValue) -> bool {
    match sensitive {
        BoolValue::Boolean(value) => *value,
        _ => false,
    }
}

fn mask_sensitive(value: Value, sensitive: &BoolValue) -> Value {
    match (value.clone(), sensitive) {
        (Value::Object(value_map), _) => Value::Object(mask_sensitive_map(value_map, sensitive)),
        (Value::Array(value_array), BoolValue::Array(sensitive_array)) => {
            Value::Array(mask_sensitive_array(value_array, sensitive_array))
        }
        _ => {
            if is_sensitive(sensitive) {
                Value::Sensitive
            } else {
                value
            }
        }
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
        *value = mask_sensitive(value.clone(), &get_child_sensitive(sensitive, key));
    }
    value_map
}

impl Change {
    #[must_use]
    pub fn from_raw(raw: RawResourceChange) -> Self {
        let before_sensitive = raw
            .change
            .before_sensitive
            .as_ref()
            .unwrap_or(&BoolValue::Boolean(false));
        let after_sensitive = raw
            .change
            .after_sensitive
            .as_ref()
            .unwrap_or(&BoolValue::Boolean(false));

        let raw_before = raw.change.before.as_ref();
        let unmasked_before = raw_before.map(value_map_from_raw);
        let before = unmasked_before.map(|before| mask_sensitive_map(before, before_sensitive));

        let raw_after = raw.change.after.as_ref();
        let unmasked_after = raw_after.map(value_map_from_raw);
        let after = unmasked_after.map(|after| mask_sensitive_map(after, after_sensitive));

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

#[derive(Serialize, Debug, PartialEq)]
pub struct Plan {
    changes: Vec<Change>,
    unique_actions: Vec<Action>,
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

        Plan {
            changes,
            unique_actions,
            raw,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Data {
    pub plans: std::collections::HashMap<String, Plan>,
}

impl Data {
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
        Ok(Data { plans })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils;

    pub enum PlanType {
        Create,
        Delete,
        DeleteCreate,
        NoOp,
        NoResources,
        Sensitive,
        Update,
    }

    fn get_test_data_plans() -> Vec<PlanType> {
        return vec![
            PlanType::Create,
            PlanType::Delete,
            PlanType::DeleteCreate,
            PlanType::NoOp,
            PlanType::NoResources,
            PlanType::Sensitive,
            PlanType::Update,
        ];
    }

    fn get_test_data_files() -> Vec<String> {
        let mut files = Vec::new();
        for plan_type in get_test_data_plans() {
            files.push(get_test_plan_file(&plan_type));
        }
        return files;
    }

    pub fn get_test_data() -> Data {
        let mut plans: std::collections::HashMap<String, Plan> = std::collections::HashMap::new();

        for plan_type in get_test_data_plans() {
            plans.insert(get_test_plan_file(&plan_type), get_test_plan(&plan_type));
        }

        return Data { plans };
    }

    pub fn get_test_plan_file(plan_type: &PlanType) -> String {
        let folder = match plan_type {
            PlanType::Create => "create",
            PlanType::Delete => "delete",
            PlanType::DeleteCreate => "delete-create",
            PlanType::NoOp => "no-op",
            PlanType::NoResources => "no-resources",
            PlanType::Sensitive => "sensitive",
            PlanType::Update => "update",
        };
        return utils::test::get_test_data_file_path(&format!(
            "plans/{folder}/terraform.tfplan.json"
        ));
    }

    pub fn get_test_plan(plan_type: &PlanType) -> Plan {
        let json = get_test_plan_json(plan_type);
        let raw = RawPlan::from_str(&json).unwrap();
        return Plan::from_raw(raw);
    }

    pub fn get_test_plan_json(plan_type: &PlanType) -> String {
        let file = get_test_plan_file(plan_type);
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
        }
    }

    mod raw_plan {
        use super::*;

        mod from_str {
            use super::*;

            macro_rules!tests {
                ($($name:ident, $plan_type:expr)*) => {
                    $(
                        #[test]
                        fn $name() {
                            get_test_plan(&$plan_type);
                        }
                    )*
                };
            }

            tests! {
                create, PlanType::Create
                delete, PlanType::Delete
                delete_create, PlanType::DeleteCreate
                no_op, PlanType::NoOp
                no_resources, PlanType::NoResources
                update, PlanType::Update
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
                let files = vec!["tests/data/plans/*/terraform.tfplan.json".to_string()];
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
