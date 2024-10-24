use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(std::collections::HashMap<String, Value>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ResourceChangeChangeAction {
    Create,
    Read,
    Update,
    Delete,
    NoOp,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResourceChangeChange {
    actions: Vec<ResourceChangeChangeAction>,
    before: Option<std::collections::HashMap<String, Option<Value>>>,
    after: std::collections::HashMap<String, Option<Value>>,
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
    resource_changes: Vec<ResourceChange>,
    // configuration
    timestamp: String,
    errored: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Data {
    pub plans: std::collections::HashMap<String, Plan>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils;

    pub enum TestPlan {
        Add,
    }

    pub fn get_test_data() -> Data {
        let mut plans = std::collections::HashMap::new();
        plans.insert("add".to_string(), get_test_plan(TestPlan::Add));

        return Data { plans };
    }

    pub fn get_test_plan(plan: TestPlan) -> Plan {
        match plan {
            TestPlan::Add => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: vec![ResourceChange {
                    address: "random_pet.example".to_string(),
                    mode: "managed".to_string(),
                    type_: "random_pet".to_string(),
                    name: "example".to_string(),
                    provider_name: "registry.terraform.io/hashicorp/random".to_string(),
                    change: ResourceChangeChange {
                        actions: vec![ResourceChangeChangeAction::Create],
                        before: None,
                        after: {
                            let mut map = std::collections::HashMap::new();
                            map.insert("keepers".to_string(), None);
                            map.insert("length".to_string(), Some(Value::Integer(2)));
                            map.insert("prefix".to_string(), None);
                            map.insert(
                                "separator".to_string(),
                                Some(Value::String("-".to_string())),
                            );

                            map
                        },
                    },
                }],
                timestamp: "2024-10-22T22:43:45Z".to_string(),
                errored: false,
            },
        }
    }

    pub fn get_test_plan_json(plan: TestPlan) -> String {
        let path = match plan {
            TestPlan::Add => "plans/add/terraform.tfplan.json",
        };
        return utils::test::get_test_data_file_contents(path);
    }

    #[test]
    fn test_deserialize() {
        let expected = get_test_plan(TestPlan::Add);

        let raw = get_test_plan_json(TestPlan::Add);
        let result = serde_json::from_str::<Plan>(&raw).unwrap();

        assert_eq!(expected, result);
    }
}
