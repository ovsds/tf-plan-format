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
    #[serde(rename = "no-op")]
    NoOp,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResourceChangeChange {
    actions: Vec<ResourceChangeChangeAction>,
    before: Option<std::collections::HashMap<String, Option<Value>>>,
    after: Option<std::collections::HashMap<String, Option<Value>>>,
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

#[derive(Serialize, Debug, PartialEq)]
pub struct Data {
    pub plans: std::collections::HashMap<String, Plan>,
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

    pub fn get_test_data() -> Data {
        let mut plans = std::collections::HashMap::new();
        plans.insert("create".to_string(), get_test_plan(&PlanType::Create));

        return Data { plans };
    }

    pub fn get_test_plan(plan_type: &PlanType) -> Plan {
        match plan_type {
            PlanType::Create => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: Some(vec![ResourceChange {
                    address: "terraform_data.foo-bar".to_string(),
                    mode: "managed".to_string(),
                    type_: "terraform_data".to_string(),
                    name: "foo-bar".to_string(),
                    provider_name: "terraform.io/builtin/terraform".to_string(),
                    change: ResourceChangeChange {
                        actions: vec![ResourceChangeChangeAction::Create],
                        before: None,
                        after: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert("input".to_string(), Some(Value::String("foo".to_string())));
                            map.insert("triggers_replace".to_string(), None);
                            map
                        }),
                    },
                }]),
                timestamp: "2024-10-25T21:40:18Z".to_string(),
                errored: false,
            },
            PlanType::Delete => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: Some(vec![ResourceChange {
                    address: "terraform_data.foo-bar".to_string(),
                    mode: "managed".to_string(),
                    type_: "terraform_data".to_string(),
                    name: "foo-bar".to_string(),
                    provider_name: "terraform.io/builtin/terraform".to_string(),
                    change: ResourceChangeChange {
                        actions: vec![ResourceChangeChangeAction::Delete],
                        before: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "id".to_string(),
                                Some(Value::String(
                                    "96202d3f-5e6b-8c7f-8e5a-7d1599601bd8".to_string(),
                                )),
                            );
                            map.insert("input".to_string(), Some(Value::String("foo".to_string())));
                            map.insert(
                                "output".to_string(),
                                Some(Value::String("foo".to_string())),
                            );
                            map.insert("triggers_replace".to_string(), None);
                            map
                        }),
                        after: None,
                    },
                }]),
                timestamp: "2024-10-25T21:40:17Z".to_string(),
                errored: false,
            },
            PlanType::DeleteCreate => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: Some(vec![ResourceChange {
                    address: "null_resource.foo-bar".to_string(),
                    mode: "managed".to_string(),
                    type_: "null_resource".to_string(),
                    name: "foo-bar".to_string(),
                    provider_name: "registry.terraform.io/hashicorp/null".to_string(),
                    change: ResourceChangeChange {
                        actions: vec![
                            ResourceChangeChangeAction::Delete,
                            ResourceChangeChangeAction::Create,
                        ],
                        before: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "triggers".to_string(),
                                Some(Value::Object({
                                    let mut map = std::collections::HashMap::new();
                                    map.insert(
                                        "always_run".to_string(),
                                        Value::String("2024-10-25T21:40:19Z".to_string()),
                                    );
                                    map
                                })),
                            );
                            map.insert(
                                "id".to_string(),
                                Some(Value::String("4525788878524015586".to_string())),
                            );
                            map
                        }),
                        after: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "triggers".to_string(),
                                Some(Value::Object(std::collections::HashMap::new())),
                            );
                            map
                        }),
                    },
                }]),
                timestamp: "2024-10-25T21:40:20Z".to_string(),
                errored: false,
            },
            PlanType::NoOp => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: Some(vec![ResourceChange {
                    address: "terraform_data.foo-bar".to_string(),
                    mode: "managed".to_string(),
                    type_: "terraform_data".to_string(),
                    name: "foo-bar".to_string(),
                    provider_name: "terraform.io/builtin/terraform".to_string(),
                    change: ResourceChangeChange {
                        actions: vec![ResourceChangeChangeAction::NoOp],
                        before: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "id".to_string(),
                                Some(Value::String(
                                    "0f61b5b9-e9e3-1625-f62b-501a232653f9".to_string(),
                                )),
                            );
                            map.insert("input".to_string(), Some(Value::String("foo".to_string())));
                            map.insert(
                                "output".to_string(),
                                Some(Value::String("foo".to_string())),
                            );
                            map.insert("triggers_replace".to_string(), None);
                            map
                        }),
                        after: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "id".to_string(),
                                Some(Value::String(
                                    "0f61b5b9-e9e3-1625-f62b-501a232653f9".to_string(),
                                )),
                            );
                            map.insert("input".to_string(), Some(Value::String("foo".to_string())));
                            map.insert(
                                "output".to_string(),
                                Some(Value::String("foo".to_string())),
                            );
                            map.insert("triggers_replace".to_string(), None);
                            map
                        }),
                    },
                }]),
                timestamp: "2024-10-25T21:40:18Z".to_string(),
                errored: false,
            },
            PlanType::NoResources => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: None,
                timestamp: "2024-10-25T21:40:18Z".to_string(),
                errored: false,
            },
            PlanType::Update => Plan {
                format_version: "1.2".to_string(),
                terraform_version: "1.7.5".to_string(),
                resource_changes: Some(vec![ResourceChange {
                    address: "terraform_data.foo-bar".to_string(),
                    mode: "managed".to_string(),
                    type_: "terraform_data".to_string(),
                    name: "foo-bar".to_string(),
                    provider_name: "terraform.io/builtin/terraform".to_string(),
                    change: ResourceChangeChange {
                        actions: vec![ResourceChangeChangeAction::Update],
                        before: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "id".to_string(),
                                Some(Value::String(
                                    "72285066-beaf-bd58-0c9f-0c5e7ae166a2".to_string(),
                                )),
                            );
                            map.insert("input".to_string(), Some(Value::String("foo".to_string())));
                            map.insert(
                                "output".to_string(),
                                Some(Value::String("foo".to_string())),
                            );
                            map.insert("triggers_replace".to_string(), None);
                            map
                        }),
                        after: Some({
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "id".to_string(),
                                Some(Value::String(
                                    "72285066-beaf-bd58-0c9f-0c5e7ae166a2".to_string(),
                                )),
                            );
                            map.insert("input".to_string(), Some(Value::String("bar".to_string())));
                            map.insert("triggers_replace".to_string(), None);
                            map
                        }),
                    },
                }]),
                timestamp: "2024-10-25T22:14:16Z".to_string(),
                errored: false,
            },
        }
    }

    pub fn get_test_plan_json(plan_type: &PlanType) -> String {
        let path = match plan_type {
            PlanType::Create => "plans/create/terraform.tfplan.json",
            PlanType::Delete => "plans/delete/terraform.tfplan.json",
            PlanType::DeleteCreate => "plans/delete-create/terraform.tfplan.json",
            PlanType::NoOp => "plans/no-op/terraform.tfplan.json",
            PlanType::NoResources => "plans/no-resources/terraform.tfplan.json",
            PlanType::Update => "plans/update/terraform.tfplan.json",
        };
        return utils::test::get_test_data_file_contents(path);
    }

    macro_rules! deserialize_tests {
        ($($name:ident, $plan_type:expr)*) => {
            $(
                #[test]
                fn $name() {
                    let expected = get_test_plan(&$plan_type);

                    let raw = get_test_plan_json(&$plan_type);
                    let result = serde_json::from_str::<Plan>(&raw).unwrap();

                    assert_eq!(expected, result);
                }
            )*
        };
    }

    deserialize_tests! {
        deserialize_create_plan, PlanType::Create
        deserialize_delete_plan, PlanType::Delete
        deserialize_delete_create_plan, PlanType::DeleteCreate
        deserialize_no_op_plan, PlanType::NoOp
        deserialize_no_resources_plan, PlanType::NoResources
        deserialize_update_plan, PlanType::Update
    }
}
