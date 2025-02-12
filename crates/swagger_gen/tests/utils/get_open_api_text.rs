use std::{env::current_dir, error::Error, fs::File, io::Read};

pub fn get_open_api_text(filename: Option<&str>) -> Result<String, Box<dyn Error>> {
    let json_dir = current_dir()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(filename.unwrap_or("3.1.0.json"));

    if !json_dir.exists() {
        panic!("文件不存在");
    }

    let mut file = File::open(json_dir).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    Ok(buffer)
}
