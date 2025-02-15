use std::collections::HashMap;

pub fn get_utils_files() -> HashMap<String, String> {
    let mut v = HashMap::<String, String>::new();
    v.insert("loop.ts".to_string(), include_str!("loop.txt").to_string());
    v.insert("request.ts".to_string(), include_str!("request.txt").to_string());
    v.insert("table.ts".to_string(), include_str!("table.txt").to_string());
    v.insert("tree-map.ts".to_string(), include_str!("tree-map.txt").to_string());
    v.insert("tree-utils.ts".to_string(), include_str!("tree-utils.txt").to_string());
    v
}
