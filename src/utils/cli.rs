pub struct CommandError {
    pub message: &'static str,
    pub exit_code: i32,
}
