use std::collections::HashMap;

use crate::model::PathItemObject;

use super::get_tags_from_path_item;

pub fn get_tags_from_paths(paths: &HashMap<String, PathItemObject>) -> Vec<&String> {
    paths
        .iter()
        .flat_map(|(_, item)| get_tags_from_path_item(item))
        .collect::<Vec<&String>>()
}
