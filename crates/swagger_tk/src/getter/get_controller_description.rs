use crate::model::OpenAPIObject;

/// 获取某个控制器的描述
pub fn get_controller_description<'a>(
    open_api: &'a OpenAPIObject,
    name: &str,
) -> Option<&'a String> {
    open_api
        .tags
        .as_ref()?
        .iter()
        .find(|&m| m.name.eq(name))?
        .description
        .as_ref()
}
