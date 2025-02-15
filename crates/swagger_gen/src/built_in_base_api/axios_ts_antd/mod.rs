use std::collections::HashMap;

pub fn get_base_files() -> HashMap<String, String> {
    let mut result = HashMap::<String, String>::new();
    result.insert("BaseService.ts".to_string(), include_str!("BaseService.txt").to_string());
    result.insert("ErrorHandler.ts".to_string(), include_str!("ErrorHandler.txt").to_string());
    result.insert("ErrorHandlerImp.ts".to_string(), include_str!("ErrorHandlerImp.txt").to_string());
    result
}
