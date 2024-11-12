pub const TEST_DATA_FOLDER_PATH: &str = "tests/data";

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

#[must_use]
pub fn get_plan_files() -> Vec<std::string::String> {
    vec![
        get_test_data_file_path("plans/create/terraform.tfplan.json"),
        get_test_data_file_path("plans/delete/terraform.tfplan.json"),
        get_test_data_file_path("plans/delete-create/terraform.tfplan.json"),
        get_test_data_file_path("plans/no-op/terraform.tfplan.json"),
        get_test_data_file_path("plans/no-resources/terraform.tfplan.json"),
        get_test_data_file_path("plans/sensitive/terraform.tfplan.json"),
        get_test_data_file_path("plans/update/terraform.tfplan.json"),
    ]
}
