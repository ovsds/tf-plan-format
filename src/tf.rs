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
enum ResourceChangeChangeAction {
    Create,
    Read,
    Update,
    Delete,
    #[serde(rename = "no-op")]
    NoOp,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResourceChangeChange {
    actions: Vec<ResourceChangeChangeAction>,
    before: Option<ValueMap>,
    after: Option<ValueMap>,
    // after_unknown
    // before_sensitive
    // after_sensitive
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResourceChange {
    address: String,
    mode: String,
    #[serde(rename = "type")]
    type_: String,
    name: String,
    provider_name: String,
    change: ResourceChangeChange,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Plan {
    format_version: String,
    terraform_version: String,
    // planned_values
    resource_changes: Option<Vec<ResourceChange>>,
    // configuration
    timestamp: String,
    errored: bool,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl FromStr for Plan {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<Plan>(s) {
            Ok(plan) => Ok(plan),
            Err(e) => Err(Error {
                message: format!("Failed to parse plan. {e}"),
            }),
        }
    }
}

impl Plan {
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let raw_file = std::fs::read_to_string(path).map_err(|e| Error {
            message: format!("Failed to read file({path}). {e}"),
        })?;
        Plan::from_str(&raw_file).map_err(|e| Error {
            message: format!("Failed to parse file({path}). {}", e.message),
        })
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Data {
    pub plans: std::collections::HashMap<String, Plan>,
}

impl Data {
    /// # Errors
    /// Returns an error if any of the files cannot be read or parsed
    pub fn from_files(paths: &[String]) -> Result<Self, Error> {
        let mut plans = std::collections::HashMap::new();
        for path_glob in paths {
            let glob = glob::glob(path_glob).map_err(|e| Error {
                message: format!("Failed to read file({path_glob}), invalid glob. {e}"),
            })?;

            let mut file_count = 0;
            for path in glob {
                let path_buf = path.map_err(|e| Error {
                    message: format!("Failed to read file({path_glob}). {e}"),
                })?;
                let Some(path) = path_buf.to_str() else {
                    return Err(Error {
                        message: format!("Failed to read file({path_glob}), invalid path."),
                    });
                };
                let plan = Plan::from_file(path)?;
                plans.insert(path.to_string(), plan);

                file_count += 1;
            }

            if file_count == 0 {
                return Err(Error {
                    message: format!("Failed to read file({path_glob}). No files found."),
                });
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
                    "Failed to parse file(tests/data/plans/artificial/invalid.json). Failed to parse plan. missing field `format_version` at line 3 column 1"
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
                    "Failed to read file(invalid path). No files found."
                );
            }
        }
    }
}
