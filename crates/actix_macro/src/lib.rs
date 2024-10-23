// pub(crate) fn add_service<T>(container: &AppContainer<_>, service: T) {
//     let app = container.get_app();
//     if let Some(app) = app {
//         app.service(service);
//     }
// }

// #[proc_macro_derive(Service)]
// pub fn service_derive(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as ItemStruct);
//     let name = &input.ident;
//     let gen = quote::quote! {
//         impl #name {
//             pub fn new() -> Self {
//                 #name
//             }
//         }
//     };
//     gen.into()
// }

// #[macro_export]
// macro_rules! fun_gen {
//     ($name:ident) => {
//         #[actix_web::get("/test")]
//         pub async fn $name() -> String {
//             "hello".to_string()
//         }
//     };
// }

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn handle(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    println!("name: {}", &input.sig.ident);

    let gen = quote::quote! {
        #input
    };
    gen.into()
}
