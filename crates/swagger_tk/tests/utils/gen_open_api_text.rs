use std::{env::current_dir, error::Error, fs::File, io::Read};

/// 返回要测试的文档内容
#[allow(dead_code)]
pub fn get_open_api_text(filename: Option<&str>) -> Result<String, Box<dyn Error>> {
    let root_dir = current_dir()?;
    let filename = filename.unwrap_or("3.1.0.json");
    let json_dir = root_dir.parent().unwrap();
    let json_dir = json_dir.parent().unwrap();
    let json_dir = json_dir.join(filename);

    if !json_dir.exists() {
        panic!("文件不存在");
    }
    let mut file = File::open(json_dir).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    Ok(buffer)
}
