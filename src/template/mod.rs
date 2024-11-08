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
        Engine::Tera => tera::render(data, template),
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
            let template = "{%- for name, plan in data.plans -%}\n{{ name }}\n{% endfor %}";
            let result = render(&Engine::Tera, &data, template).unwrap();

            let expected = utils::test::get_test_data_file_contents("renders/tera/custom.md");

            assert_eq!(expected, result);
        }
    }
}
