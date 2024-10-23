use actix_web::App;

pub struct AppContainer<'a, T: 'a> {
    app: Option<Box<&'a App<T>>>,
}

impl<'a, T: 'a> AppContainer<'a, T> {
    pub fn new() -> Self {
        AppContainer { app: None }
    }

    pub fn get_app(&self) -> Option<&App<T>> {
        self.app.as_ref().map(|app| **app)
    }

    pub fn set_app(&mut self, app: &'a App<T>) -> &mut Self {
        self.app = Some(Box::new(app));
        self
    }
}
