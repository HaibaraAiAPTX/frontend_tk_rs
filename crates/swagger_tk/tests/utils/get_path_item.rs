use swagger_tk::model::{OpenAPIObject, PathItemObject};

#[allow(dead_code)]
pub fn get_path_item<'a>(
    data: &'a OpenAPIObject,
    filter_path: &'a str,
) -> Option<&'a PathItemObject> {
    let paths = data.paths.as_ref().unwrap();
    let (_path, path_item) = paths
        .iter()
        .find(|&(path, _path_item)| path == filter_path)
        .unwrap();
    Some(path_item)
}
