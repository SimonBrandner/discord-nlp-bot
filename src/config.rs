#[derive(serde::Deserialize)]
pub struct Configuration {
    pub discord_token: String,
}

fn read_file_as_string(path: String) -> String {
    let json_file_path = std::path::Path::new(&path);
    let file = std::fs::File::open(json_file_path).expect("Failed to open file");
    return std::io::read_to_string(file).expect("Failed to read file");
}

pub fn read_configuration_from_file(path: String) -> Configuration {
    let json_str = read_file_as_string(path);
    return serde_json::from_str(&json_str).expect("Failed to parse JSON");
}
