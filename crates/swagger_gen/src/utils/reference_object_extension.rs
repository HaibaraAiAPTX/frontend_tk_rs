use swagger_tk::model::ReferenceObject;

pub trait ReferenceObjectExtension {
    fn get_type_name(&self) -> String;
}

impl ReferenceObjectExtension for ReferenceObject {
    fn get_type_name(&self) -> String {
        self.r#ref.split('/').last().unwrap().to_string()
    }
}
