use crate::model::OpenAPIObject;

pub fn get_tags_from_open_api(data: &OpenAPIObject) -> Option<Vec<&String>> {
    data.tags
        .as_ref()
        .map(|tags| tags.iter().map(|tag| &tag.name).collect())
}
