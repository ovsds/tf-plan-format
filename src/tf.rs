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
        plans.insert("delete".to_string(), get_test_plan(&PlanType::Delete));
        plans.insert(
            "delete-create".to_string(),
            get_test_plan(&PlanType::DeleteCreate),
        );
        plans.insert("no-op".to_string(), get_test_plan(&PlanType::NoOp));
        plans.insert(
            "no-resources".to_string(),
            get_test_plan(&PlanType::NoResources),
        );
        plans.insert("update".to_string(), get_test_plan(&PlanType::Update));

        return Data { plans };
    }

    pub fn get_test_plan(plan_type: &PlanType) -> Plan {
        let json = get_test_plan_json(plan_type);
        return serde_json::from_str::<Plan>(&json).unwrap();
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
                    get_test_plan(&$plan_type);
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
