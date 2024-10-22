use uuid::Uuid;

pub struct Project {
  pub id: Uuid,

  pub name: String,

  pub description: Option<String>,

  pub swagger_url: Option<String>,

  pub file_path: Option<String>,
}
