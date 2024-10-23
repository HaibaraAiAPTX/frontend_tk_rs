pub struct AddProjectDto {
    pub name: String,
    pub description: Option<String>,
    pub swagger_url: Option<String>,
    pub file_path: Option<String>,
}
