use crate::{getter::get_schema_name_from_ref, model::SchemaEnum};

impl SchemaEnum {
    pub fn get_ref_full_name(&self) -> Option<&String> {
        match self {
            SchemaEnum::Ref(reference_object) => Some(&reference_object.r#ref),
            _ => None,
        }
    }

    pub fn get_ref_schema_name(&self) -> Option<&str> {
        self.get_ref_full_name()
            .and_then(|v| get_schema_name_from_ref(v))
    }

    pub fn get_object_property(&self, name: &str) -> Option<&SchemaEnum> {
        match self {
            SchemaEnum::Object(o) => o.properties.as_ref().and_then(|data| data.get(name)),
            _ => None,
        }
    }

    pub fn get_array_item_ref_schema_name(&self) -> Option<&str> {
        match self {
            SchemaEnum::Array(a) => match &*a.items {
                SchemaEnum::Ref(r) => get_schema_name_from_ref(&r.r#ref),
                _ => None,
            },
            _ => None,
        }
    }
}
