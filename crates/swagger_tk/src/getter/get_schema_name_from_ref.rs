pub fn get_schema_name_from_ref(full_ref: &String) -> Option<&str> {
    full_ref.split("/").last()
}
