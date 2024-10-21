use std::{
    env::current_dir,
    fs::File,
    io::{BufReader, Read},
};

pub fn get_json_file(name: &str) -> Option<String> {
    let file_dir = current_dir().expect("获取工作目录失败");
    let file_dir = file_dir.join(name);
    if !file_dir.exists() {
        println!("文件不存在: {}", file_dir.display());
        return None;
    }

    let file = File::open(file_dir).expect("打开文件失败");
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf).expect("读取文件失败");
    Some(buf)
}
