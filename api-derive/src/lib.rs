use convert_case::{Case, Casing as _};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    let route_path = parse_macro_input!(attr as LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;

    let path_lit = route_path.value();

    let struct_name = Ident::new(&fn_name.to_string().to_case(Case::Pascal), fn_name.span());

    let expanded = quote! {
        #vis struct #struct_name;

        impl crate::route::Router for #struct_name {
            fn path() -> &'static str {
                #path_lit
            }

            fn handler() -> impl poem::IntoEndpoint {
                #[poem::handler]
                #input_fn
                #fn_name
            }

            fn method() -> poem::http::Method {
                poem::http::Method::GET
            }
        }
    };

    TokenStream::from(expanded)
}
