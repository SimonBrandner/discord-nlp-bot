use std::io::Error;

pub fn read_file_as_string(path: &String) -> Result<String, Error> {
    let json_file_path = std::path::Path::new(path);
    let file = std::fs::File::open(json_file_path)?;
    std::io::read_to_string(file)
}
