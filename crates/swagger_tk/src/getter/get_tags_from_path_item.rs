use std::collections::HashSet;

use crate::model::PathItemObject;

pub fn get_tags_from_path_item(op: &PathItemObject) -> Vec<&String> {
    vec![
        &op.get,
        &op.put,
        &op.post,
        &op.delete,
        &op.options,
        &op.head,
        &op.patch,
        &op.trace,
    ]
    .iter()
    .filter_map(|method| method.as_ref().and_then(|m| m.tags.as_ref()))
    .flat_map(|tags| tags.iter())
    .collect::<HashSet<&String>>()
    .iter()
    .map(|&v| v)
    .collect()
}
