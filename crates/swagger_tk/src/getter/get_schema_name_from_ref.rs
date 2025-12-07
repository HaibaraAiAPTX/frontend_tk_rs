pub fn get_schema_name_from_ref(full_ref: &str) -> Option<&str> {
    full_ref.split("/").last()
}
