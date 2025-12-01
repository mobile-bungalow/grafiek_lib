use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

// TODO: Support fieldless enums as config values.
// They should map to a labeled list of integer options in the metadata.
// Example:
//   #[derive(ConfigSchema)]
//   enum BlendMode { Add, Multiply, Screen }
// Would register as an i32 slot with metadata containing the variant names.

/// Derive InputSchema for a struct
///
/// Usage:
/// ```ignore
/// #[derive(InputSchema)]
/// struct MyInputs {
///     value: f32,
///     count: i32,
/// }
/// ```
#[proc_macro_derive(InputSchema)]
pub fn derive_input_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

/// Derive OutputSchema for a struct
///
/// Usage:
/// ```ignore
/// #[derive(OutputSchema)]
/// struct MyOutputs {
///     result: f32,
/// }
/// ```
#[proc_macro_derive(OutputSchema)]
pub fn derive_output_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Output) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

/// Derive ConfigSchema for a struct
///
/// Usage:
/// ```ignore
/// #[derive(ConfigSchema)]
/// struct MyConfig {
///     threshold: f32,
/// }
/// ```
#[proc_macro_derive(ConfigSchema)]
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

fn derive_schema_impl(input: DeriveInput, kind: SchemaKind) -> syn::Result<TokenStream> {
    let name = &input.ident;

    // Get struct fields
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
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let field_count = field_names.len();
    let field_name_strs: Vec<_> = field_names
        .iter()
        .map(|n| n.as_ref().unwrap().to_string())
        .collect();
    let field_indices: Vec<_> = (0..field_count).collect();

    // Base Schema impl
    let schema_impl = quote! {
        impl grafiek_engine::traits::Schema for #name {
            fn metadata(field: &str) -> grafiek_engine::registry::Metadata {
                grafiek_engine::registry::Metadata {}
            }

            fn fields() -> &'static [&'static str] {
                &[#(#field_name_strs),*]
            }

            fn len() -> usize {
                #field_count
            }
        }
    };

    // Default impl
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

    // Kind-specific impl
    let kind_impl = match kind {
        SchemaKind::Input => quote! {
            impl grafiek_engine::traits::InputSchema for #name {
                fn register(registry: &mut grafiek_engine::registry::SignatureRegistery) {
                    #(
                        registry.add_input(
                            <#field_types as grafiek_engine::value::AsValueType>::value_type(),
                            grafiek_engine::registry::SlotMetadata {
                                name: #field_name_strs.to_owned(),
                            },
                        );
                    )*
                }

                fn try_extract(inputs: grafiek_engine::value::Inputs) -> grafiek_engine::error::Result<Self> {
                    Ok(Self {
                        #(
                            #field_names: *inputs.get::<#field_types>(#field_indices)?,
                        )*
                    })
                }
            }
        },
        SchemaKind::Output => quote! {
            impl grafiek_engine::traits::OutputSchema for #name {
                fn register(registry: &mut grafiek_engine::registry::SignatureRegistery) {
                    #(
                        registry.add_output(
                            <#field_types as grafiek_engine::value::AsValueType>::value_type(),
                            grafiek_engine::registry::SlotMetadata {
                                name: #field_name_strs.to_owned(),
                            },
                        );
                    )*
                }

                fn try_write(&self, mut outputs: grafiek_engine::value::Outputs) -> grafiek_engine::error::Result<()> {
                    #(
                        outputs.set(#field_indices, self.#field_names.clone())?;
                    )*
                    Ok(())
                }
            }
        },
        SchemaKind::Config => quote! {
            impl grafiek_engine::traits::ConfigSchema for #name {
                fn register(registry: &mut grafiek_engine::registry::SignatureRegistery) {
                    #(
                        registry.add_config(
                            <#field_types as grafiek_engine::value::AsValueType>::value_type(),
                            grafiek_engine::registry::SlotMetadata {
                                name: #field_name_strs.to_owned(),
                            },
                        );
                    )*
                }
            }
        },
    };

    Ok(quote! {
        #schema_impl
        #default_impl
        #kind_impl
    }
    .into())
}
