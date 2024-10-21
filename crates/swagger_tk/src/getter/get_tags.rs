use crate::model::OpenAPIObject;

use super::get_tags_from_paths;

pub fn get_tags(data: &OpenAPIObject) -> Vec<&String> {
    if let Some(tags) = &data.tags {
        tags.iter().map(|tag| &tag.name).collect()
    } else if let Some(paths) = &data.paths {
        get_tags_from_paths(paths)
    } else {
        vec![]
    }
}
