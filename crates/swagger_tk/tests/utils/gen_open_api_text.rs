use std::{env::current_dir, error::Error, fs::File, io::Read};

/// 返回要测试的文档内容
pub fn get_open_api_text() -> Result<String, Box<dyn Error>> {
    let root_dir = current_dir()?;
    let json_dir = root_dir.parent().unwrap();
    let json_dir = json_dir.parent().unwrap();
    let json_dir = json_dir.join("3.1.0.json");

    if !json_dir.exists() {
        panic!("文件不存在");
    }
    let mut file = File::open(json_dir).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    Ok(buffer)
}
