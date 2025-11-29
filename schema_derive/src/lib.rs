use proc_macro::TokenStream;
use syn::{Ident, parse_macro_input};

struct SchemaArgs {
    ty: Ident,
}

impl syn::parse::Parse for SchemaArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Ident = input.parse()?;
        let valid_streams = ["Input", "Output", "Config"];
        if !valid_streams.iter().any(|s| ty == s) {
            return Err(syn::Error::new(
                ty.span(),
                format!("Argument must be one of: {}", valid_streams.join(", ")),
            ));
        }
        Ok(SchemaArgs { ty })
    }
}

#[proc_macro_derive(Schema)]
pub fn derive_scehma(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SchemaArgs);

    TokenStream::from(
        syn::Error::new(
            input.ty.span(),
            "Only structs with named fields can derive `Schema`",
        )
        .to_compile_error(),
    )
}
