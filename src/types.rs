use std::convert::Into;

#[derive(Debug)]
pub enum ErrorType {
    Default,
    Command { exit_code: exitcode::ExitCode },
}

#[derive(Debug)]
pub struct Error {
    pub error_type: ErrorType,
    message: String,
    source: Option<Box<dyn std::error::Error + Sync + Send>>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.error_type {
            ErrorType::Command { exit_code: _ } => write!(f, "{}", self.full_message()),
            ErrorType::Default => write!(f, "{}", self.message),
        }
    }
}

impl Error {
    #[must_use]
    pub fn default(message: String) -> Self {
        Self {
            error_type: ErrorType::Default,
            message,
            source: None,
        }
    }

    #[must_use]
    pub fn chain<T: Into<Box<dyn std::error::Error + Sync + Send>>>(
        message: String,
        source: T,
    ) -> Self {
        Self {
            error_type: ErrorType::Default,
            message,
            source: Some(source.into()),
        }
    }

    #[must_use]
    pub fn command<T: Into<Box<dyn std::error::Error + Sync + Send>>>(
        message: String,
        exit_code: exitcode::ExitCode,
        source: T,
    ) -> Self {
        Self {
            error_type: ErrorType::Command { exit_code },
            message,
            source: Some(source.into()),
        }
    }

    #[must_use]
    pub fn full_message(&self) -> String {
        use std::error::Error;

        let mut message = self.message.clone();
        let mut cause = self.source();
        while let Some(error) = cause {
            message = format!("{message}. {error}");
            cause = error.source();
        }

        message
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|c| &**c as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error {
        use super::*;

        #[test]
        fn display() {
            let error = Error::default("message".to_string());
            assert_eq!("message", format!("{}", error));
        }

        #[test]
        fn chain() {
            let error = Error::chain("message".to_string(), tera::Error::msg("source"));
            assert_eq!("message. source", format!("{}", error.full_message()));
        }

        #[test]
        fn chain_multiple() {
            let grandparent = Error::default("grandparent".to_string());
            let parent = Error::chain("parent".to_string(), grandparent);
            let error = Error::chain("child".to_string(), parent);

            assert_eq!(
                "child. parent. grandparent",
                format!("{}", error.full_message())
            );
        }

        #[test]
        fn command() {
            let error = Error::command("message".to_string(), 1, tera::Error::msg("source"));
            assert_eq!("message. source", format!("{}", error.full_message()));
        }
    }
}
