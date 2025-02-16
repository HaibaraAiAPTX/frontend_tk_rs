use crate::{getter::get_schema_name_from_ref, model::MediaTypeObject};

impl MediaTypeObject {
    pub fn get_ref_schema_name(&self) -> Option<&str> {
        self.schema
            .get_ref_full_name()
            .and_then(|name| get_schema_name_from_ref(name))
    }
}
