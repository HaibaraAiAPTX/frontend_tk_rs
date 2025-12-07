use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, parse_macro_input};

#[proc_macro_attribute]
pub fn schema_base_attributes(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemStruct);
    let input_name = &input.ident;

    let expanded = match &mut input.fields {
        Fields::Named(name_fields) => {
            let attrs = input.attrs;
            let old_fields = name_fields.named.iter();

            quote! {
                #(#attrs)*
                pub struct #input_name {
                    #(#old_fields,)*

                    /// 是否可以为空
                    pub nullable: Option<bool>,

                    /// 是否只读
                    #[serde(rename = "readOnly")]
                    pub read_only: Option<bool>,

                    /// 是否只写
                    #[serde(rename = "writeOnly")]
                    pub write_only: Option<bool>,
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(&input.fields, "Unsupported struct type")
                .to_compile_error()
                .into();
        }
    };

    TokenStream::from(expanded)
}
