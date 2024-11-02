#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error {
    #[must_use]
    pub fn new(message: String) -> Self {
        Self { message }
    }

    #[must_use]
    pub fn inherit(parent: impl std::fmt::Display, message: &String) -> Self {
        Self {
            message: format!("{message}. {parent}"),
        }
    }
}

pub struct CommandError {
    pub message: String,
    pub exit_code: i32,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl CommandError {
    #[must_use]
    pub fn new(message: String, exit_code: i32) -> Self {
        Self { message, exit_code }
    }

    #[must_use]
    pub fn inherit(parent: impl std::fmt::Display, message: &String, exit_code: i32) -> Self {
        Self {
            message: format!("{message}. {parent}"),
            exit_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error {
        use super::*;

        #[test]
        fn display() {
            let error = Error {
                message: "message".to_string(),
            };

            assert_eq!("message", format!("{}", error));
        }

        #[test]
        fn new() {
            let error = Error::new("message".to_string());
            assert_eq!("message", error.message);
        }

        #[test]
        fn inherit() {
            let error = Error::new("message".to_string());
            let error = Error::inherit(error, &"inherited".to_string());
            assert_eq!("inherited. message", error.message);
        }
    }

    mod command_error {
        use super::*;

        #[test]
        fn display() {
            let error = CommandError {
                message: "message".to_string(),
                exit_code: 1,
            };

            assert_eq!("message", format!("{}", error));
        }

        #[test]
        fn new() {
            let error = CommandError::new("message".to_string(), 1);
            assert_eq!("message", error.message);
            assert_eq!(1, error.exit_code);
        }

        #[test]
        fn inherit() {
            let error = CommandError::new("message".to_string(), 1);
            let error = CommandError::inherit(error, &"inherited".to_string(), 2);
            assert_eq!("inherited. message", error.message);
            assert_eq!(2, error.exit_code);
        }
    }
}
