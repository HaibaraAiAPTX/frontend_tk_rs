use std::collections::HashMap;

use crate::model::{OpenAPIObject, PathItemObject};

use super::get_tags_from_path_item;

pub fn get_paths_from_tag<'a>(
    open_api_object: &'a OpenAPIObject,
    tag: &'a str,
) -> HashMap<&'a String, &'a PathItemObject> {
    let mut result = HashMap::<&String, &PathItemObject>::new();

    open_api_object.paths.iter().flatten().for_each(|item| {
        let tags = get_tags_from_path_item(item.1);
        if tags.iter().any(|t| *t == tag) {
            result.insert(item.0, item.1);
        }
    });

    result
}
