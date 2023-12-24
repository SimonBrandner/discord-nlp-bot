use crate::file;

#[derive(serde::Deserialize)]
pub struct Configuration {
    pub discord_token: String,
    pub data_file_path: String,
}

pub fn read_configuration_from_file(path: String) -> Configuration {
    let json_str = file::read_file_as_string(path);
    return serde_json::from_str(&json_str).expect("Failed to parse JSON");
}
