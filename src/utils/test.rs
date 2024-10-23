const TEST_DATA_FOLDER_PATH: &str = "tests/data";

#[must_use]
pub fn get_test_data_file_path(relative_path: &str) -> std::string::String {
    let folder = std::path::Path::new(TEST_DATA_FOLDER_PATH);
    let file = folder.join(relative_path);
    return file.to_string_lossy().to_string();
}

#[must_use]
pub fn get_test_data_file_contents(relative_path: &str) -> std::string::String {
    let file = get_test_data_file_path(relative_path);
    return std::fs::read_to_string(file).unwrap();
}
