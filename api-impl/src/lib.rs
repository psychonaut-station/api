use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, ItemMod, parse_macro_input};

#[proc_macro_attribute]
pub fn endpoint(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_mod = parse_macro_input!(input as ItemMod);

    let Some((_, content)) = item_mod.content else {
        return syn::Error::new_spanned(item_mod, "expected content")
            .to_compile_error()
            .into();
    };

    let mut items = Vec::new();
    let mut handlers = Vec::new();
    let mut responses = Vec::new();

    for item in content {
        match item {
            Item::Fn(item_fn) => handlers.push(item_fn),
            Item::Enum(mut item_enum) => {
                let idx = item_enum
                    .attrs
                    .iter()
                    .position(|attr| attr.path().is_ident("response"));
                if let Some(idx) = idx {
                    item_enum.attrs.remove(idx);
                    responses.push(item_enum);
                } else {
                    items.push(Item::Enum(item_enum));
                }
            }
            _ => items.push(item),
        }
    }

    let expanded = quote! {
        #(#items)*

        #(#[derive(::poem_openapi::ApiResponse)]#responses)*

        pub struct Endpoint;

        #[::poem_openapi::OpenApi]
        impl Endpoint {
            #(#handlers)*
        }
    };

    TokenStream::from(expanded)
}
