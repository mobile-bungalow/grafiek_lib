use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

#[proc_macro_derive(EnumSchema)]
pub fn derive_schema_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Enum(data) => {
            let variants: Vec<_> = data.variants.iter().collect();
            let name = &input.ident;

            if variants.iter().any(|v| !matches!(v.fields, Fields::Unit)) {
                return syn::Error::new_spanned(
                    &input,
                    "SchemaEnum can only be derived for enums with unit variants.",
                )
                .to_compile_error()
                .into();
            }
            let variant_names: Vec<_> = variants.iter().map(|v| v.ident.clone()).collect();

            quote! {
                impl grafiek_engine::traits::SchemaEnum for #name {
                    const VARIANTS : &'static [(&str, i32)] = &[
                        #(
                            (stringify!(#variant_names), #name::#variant_names as i32),
                        )*
                    ];
                }
            }
            .into()
        }
        _ => syn::Error::new_spanned(&input, "SchemaEnum can only be derived for enums.")
            .to_compile_error()
            .into(),
    }
}

#[proc_macro_derive(InputSchema, attributes(meta))]
pub fn derive_input_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(OutputSchema, attributes(meta))]
pub fn derive_output_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Output) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(ConfigSchema, attributes(meta))]
pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Config) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

enum SchemaKind {
    Input,
    Output,
    Config,
}

/// Generate the SlotDef expression for a field, incorporating any metadata attributes
fn generate_slot_def(field: &Field, field_name_str: &str) -> syn::Result<TokenStream2> {
    let field_type = &field.ty;

    let meta_attribute = field.attrs.iter().find(|a| a.path().is_ident("meta"));

    let meta_attribtue_args = meta_attribute
        .map(|m| m.meta.require_list().map(|c| c.tokens.clone()))
        .transpose()?;

    // find first meta.
    if let Some(meta_tokens) = meta_attribtue_args {
        Ok(quote! {
            grafiek_engine::SlotDef::with_metadata(
                <#field_type as grafiek_engine::AsValueType>::VALUE_TYPE,
                #field_name_str,
                grafiek_engine::ExtendedMetadata::from(#meta_tokens),
            )
        })
    } else {
        Ok(quote! {
            grafiek_engine::SlotDef::new(
                <#field_type as grafiek_engine::AsValueType>::VALUE_TYPE,
                #field_name_str,
            )
        })
    }
}

fn derive_schema_impl(input: DeriveInput, kind: SchemaKind) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Schema can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Schema can only be derived for structs",
            ));
        }
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    let field_name_strs: Vec<_> = field_names
        .iter()
        .map(|n| n.as_ref().unwrap().to_string())
        .collect();

    let slot_defs: Vec<TokenStream2> = fields
        .iter()
        .zip(field_name_strs.iter())
        .map(|(field, name_str)| generate_slot_def(field, name_str))
        .collect::<syn::Result<_>>()?;

    let schema_impl = quote! {
        impl grafiek_engine::traits::Schema for #name {
            fn fields() -> Vec<grafiek_engine::SlotDef> {
                vec![ #( #slot_defs, )* ]
            }
        }
    };

    let default_impl = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(
                        #field_names: Default::default(),
                    )*
                }
            }
        }
    };

    let kind_impl = match kind {
        SchemaKind::Input => quote! {
            impl grafiek_engine::traits::InputSchema for #name {
                fn try_extract(inputs: grafiek_engine::Inputs) -> grafiek_engine::error::Result<Self> {
                    todo!("try_extract not yet implemented")
                }
            }
        },
        SchemaKind::Output => quote! {
            impl grafiek_engine::traits::OutputSchema for #name {
                fn try_write(&self, mut outputs: grafiek_engine::Outputs) -> grafiek_engine::error::Result<()> {
                    todo!("try_write not yet implemented")
                }
            }
        },
        SchemaKind::Config => quote! {
            impl grafiek_engine::traits::ConfigSchema for #name {}
        },
    };

    Ok(quote! {
        #schema_impl
        #default_impl
        #kind_impl
    }
    .into())
}
