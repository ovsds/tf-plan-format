use crate::types;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Option<Value>>),
    Object(std::collections::HashMap<String, Option<Value>>),
}

pub type ValueMap = std::collections::HashMap<String, Option<Value>>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceChangeChangeAction {
    Create,
    Read,
    Update,
    Delete,
    #[serde(rename = "no-op")]
    NoOp,
}

impl FromStr for ResourceChangeChangeAction {
    type Err = types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<ResourceChangeChangeAction>(format!("\"{s}\"").as_str()) {
            Ok(action) => Ok(action),
            Err(e) => Err(types::Error::new(format!("Failed to parse action. {e}"))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ResultAction {
    Create,
    DeleteCreate,
    Read,
    Update,
    Delete,
    NoOp,
    Unknown,
}

impl ResultAction {
    #[must_use]
    pub fn from_actions(actions: &[ResourceChangeChangeAction]) -> ResultAction {
        if actions.len() == 2
            && actions.contains(&ResourceChangeChangeAction::Create)
            && actions.contains(&ResourceChangeChangeAction::Delete)
        {
            return ResultAction::DeleteCreate;
        };
        if actions.len() == 1 {
            return match actions[0] {
                ResourceChangeChangeAction::Create => ResultAction::Create,
                ResourceChangeChangeAction::Read => ResultAction::Read,
                ResourceChangeChangeAction::Update => ResultAction::Update,
                ResourceChangeChangeAction::Delete => ResultAction::Delete,
                ResourceChangeChangeAction::NoOp => ResultAction::NoOp,
            };
        }
        ResultAction::Unknown
    }

    /// # Errors
    /// Returns an error if any of the actions cannot be parsed
    pub fn from_strings(actions: &Vec<String>) -> Result<ResultAction, types::Error> {
        let mut parsed_actions = Vec::new();
        for action in actions {
            match ResourceChangeChangeAction::from_str(action) {
                Ok(action) => parsed_actions.push(action),
                Err(_) => {
                    return Err(types::Error::new(format!(
                        "Failed to parse action({action})"
                    )))
                }
            }
        }
        Ok(ResultAction::from_actions(&parsed_actions))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResourceChangeChange {
    pub actions: Vec<ResourceChangeChangeAction>,
    pub before: Option<ValueMap>,
    pub after: Option<ValueMap>,
    // after_unknown
    // before_sensitive
    // after_sensitive
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResourceChange {
    pub address: String,
    // mode: String,
    #[serde(rename = "type")]
    // type_: String,
    pub name: String,
    // provider_name: String,
    pub change: ResourceChangeChange,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Plan {
    // format_version: String,
    // terraform_version: String,
    // planned_values
    pub resource_changes: Option<Vec<ResourceChange>>,
    // configuration
    // timestamp: String,
    // errored: bool,
}

impl FromStr for Plan {
    type Err = types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<Plan>(s) {
            Ok(plan) => Ok(plan),
            Err(e) => Err(types::Error::inherit(
                e,
                &"Failed to parse plan".to_string(),
            )),
        }
    }
}

impl Plan {
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: &str) -> Result<Self, types::Error> {
        let raw_file = std::fs::read_to_string(path)
            .map_err(|e| types::Error::inherit(e, &format!("Failed to read file({path})")))?;
        Plan::from_str(&raw_file)
            .map_err(|e| types::Error::inherit(e, &format!("Failed to parse file({path})")))
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
        let mut plans = std::collections::HashMap::new();
        for path_glob in paths {
            let glob = glob::glob(path_glob).map_err(|e| {
                types::Error::inherit(
                    e,
                    &format!("Failed to read file({path_glob}), invalid glob"),
                )
            })?;

            let mut file_count = 0;
            for path in glob {
                let path_buf = path.map_err(|e| {
                    types::Error::inherit(e, &format!("Failed to read file({path_glob})"))
                })?;
                let Some(path) = path_buf.to_str() else {
                    return Err(types::Error::new(format!(
                        "Failed to read file({path_glob}), invalid path"
                    )));
                };
                let plan = Plan::from_file(path)
                    .map_err(|e| types::Error::new(format!("Failed to read file({path}). {e}")))?;
                plans.insert(path.to_string(), plan);

                file_count += 1;
            }

            if file_count == 0 {
                return Err(types::Error::new(format!(
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
        Update,
    }

    fn get_test_data_plans() -> Vec<PlanType> {
        return vec![
            PlanType::Create,
            PlanType::Delete,
            PlanType::DeleteCreate,
            PlanType::NoOp,
            PlanType::NoResources,
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
        let mut plans = std::collections::HashMap::new();

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
            PlanType::Update => "update",
        };
        return utils::test::get_test_data_file_path(&format!(
            "plans/{folder}/terraform.tfplan.json"
        ));
    }

    pub fn get_test_plan(plan_type: &PlanType) -> Plan {
        let json = get_test_plan_json(plan_type);
        return Plan::from_str(&json).unwrap();
    }

    pub fn get_test_plan_json(plan_type: &PlanType) -> String {
        let file = get_test_plan_file(plan_type);
        return std::fs::read_to_string(file).unwrap();
    }

    mod result_action {
        use super::*;

        mod from_actions {
            use super::*;
            #[test]
            fn create() {
                let actions = vec![ResourceChangeChangeAction::Create];
                assert_eq!(ResultAction::from_actions(&actions), ResultAction::Create);
            }

            #[test]
            fn delete() {
                let actions = vec![ResourceChangeChangeAction::Delete];
                assert_eq!(ResultAction::from_actions(&actions), ResultAction::Delete);
            }

            #[test]
            fn delete_create() {
                let actions = vec![
                    ResourceChangeChangeAction::Create,
                    ResourceChangeChangeAction::Delete,
                ];
                assert_eq!(
                    ResultAction::from_actions(&actions),
                    ResultAction::DeleteCreate
                );
            }

            #[test]
            fn no_op() {
                let actions = vec![ResourceChangeChangeAction::NoOp];
                assert_eq!(ResultAction::from_actions(&actions), ResultAction::NoOp);
            }

            #[test]
            fn read() {
                let actions = vec![ResourceChangeChangeAction::Read];
                assert_eq!(ResultAction::from_actions(&actions), ResultAction::Read);
            }

            #[test]
            fn update() {
                let actions = vec![ResourceChangeChangeAction::Update];
                assert_eq!(ResultAction::from_actions(&actions), ResultAction::Update);
            }

            #[test]
            fn unknown() {
                let actions = vec![];
                assert_eq!(ResultAction::from_actions(&actions), ResultAction::Unknown);
            }
        }

        mod from_strings {

            use super::*;

            #[test]
            fn create() {
                assert_eq!(
                    ResultAction::from_strings(&vec!["create".to_string()]).unwrap(),
                    ResultAction::Create
                );
            }

            #[test]
            fn delete() {
                assert_eq!(
                    ResultAction::from_strings(&vec!["delete".to_string()]).unwrap(),
                    ResultAction::Delete
                );
            }

            #[test]
            fn delete_create() {
                assert_eq!(
                    ResultAction::from_strings(&vec!["create".to_string(), "delete".to_string()])
                        .unwrap(),
                    ResultAction::DeleteCreate
                );
            }

            #[test]
            fn no_op() {
                assert_eq!(
                    ResultAction::from_strings(&vec!["no-op".to_string()]).unwrap(),
                    ResultAction::NoOp
                );
            }

            #[test]
            fn read() {
                assert_eq!(
                    ResultAction::from_strings(&vec!["read".to_string()]).unwrap(),
                    ResultAction::Read
                );
            }

            #[test]
            fn update() {
                assert_eq!(
                    ResultAction::from_strings(&vec!["update".to_string()]).unwrap(),
                    ResultAction::Update
                );
            }

            #[test]
            fn unknown() {
                assert_eq!(
                    ResultAction::from_strings(&vec![]).unwrap(),
                    ResultAction::Unknown
                );
            }

            #[test]
            fn invalid() {
                let action = ResultAction::from_strings(&vec!["invalid".to_string()]);
                assert_eq!(
                    action.unwrap_err().message,
                    "Failed to parse action(invalid)"
                );
            }
        }
    }

    mod plan {
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
                let plan = Plan::from_str("invalid json");
                assert_eq!(
                    plan.unwrap_err().message,
                    "Failed to parse plan. expected value at line 1 column 1"
                );
            }
        }

        mod from_file {
            use super::*;

            #[test]
            fn full() {
                let path = utils::test::get_test_data_file_path("plans/artificial/full.json");
                Plan::from_file(&path).unwrap();
            }

            #[test]
            fn no_resource_changes() {
                let path = utils::test::get_test_data_file_path(
                    "plans/artificial/no-resource-changes.json",
                );
                Plan::from_file(&path).unwrap();
            }

            #[test]
            fn invalid_path() {
                let plan = Plan::from_file("invalid path");
                assert_eq!(
                    plan.unwrap_err().message,
                    "Failed to read file(invalid path). No such file or directory (os error 2)"
                );
            }

            #[test]
            fn invalid_json() {
                let path = utils::test::get_test_data_file_path("plans/artificial/invalid.json");
                let plan = Plan::from_file(&path);
                assert_eq!(
                    plan.unwrap_err().message,
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
                    data.unwrap_err().message,
                    "Failed to read file(*****), invalid glob. Pattern syntax error near position 2: wildcards are either regular `*` or recursive `**`"
                );
            }

            #[test]
            fn no_files() {
                let data = Data::from_files(&["invalid path".to_string()]);
                assert_eq!(
                    data.unwrap_err().message,
                    "Failed to read file(invalid path). No files found"
                );
            }
        }
    }
}
