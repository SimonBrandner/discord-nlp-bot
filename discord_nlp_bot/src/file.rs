pub fn read_file_as_string(path: &String) -> String {
    let json_file_path = std::path::Path::new(path);
    let file = std::fs::File::open(json_file_path).expect("Failed to open file");
    std::io::read_to_string(file).expect("Failed to read file")
}
