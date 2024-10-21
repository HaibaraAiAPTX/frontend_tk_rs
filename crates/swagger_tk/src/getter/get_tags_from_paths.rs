use std::collections::{HashMap, HashSet};

use crate::model::PathItemObject;

use super::get_tags_from_path_item;

pub fn get_tags_from_paths(paths: &HashMap<String, PathItemObject>) -> Vec<&String> {
    paths
        .iter()
        .map(|(_, item)| get_tags_from_path_item(item))
        .flatten()
        .collect::<HashSet<&String>>()
        .iter()
        .map(|&v| v)
        .collect()
}
