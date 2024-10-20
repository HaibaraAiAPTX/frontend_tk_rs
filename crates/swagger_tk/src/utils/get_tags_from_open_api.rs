use crate::model::OpenAPIObject;

pub fn get_tags_from_open_api(data: &OpenAPIObject) -> Option<Vec<&String>> {
    data.tags.as_ref().and_then(|tags| {
        Some(
            tags.iter()
                .filter_map(|tag| Some(&tag.name))
                .collect::<Vec<&String>>(),
        )
    })
}
