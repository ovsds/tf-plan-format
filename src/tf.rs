use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(std::collections::HashMap<String, Value>),
    Raw(serde_json::Value),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

    fn get_expected_add_plan() -> Plan {
        return Plan {
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
        };
    }

    #[test]
    fn test_deserialize() {
        let raw = utils::test::get_test_data_file_contents("plans/add/terraform.tfplan.json");
        let result = serde_json::from_str::<Plan>(&raw).unwrap();

        let expected = get_expected_add_plan();
        assert_eq!(expected, result);
    }
}
