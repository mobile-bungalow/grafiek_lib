use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

fn crate_path() -> TokenStream2 {
    let is_in_engine = std::env::var("CARGO_PKG_NAME")
        .map(|n| n != "grafiek_engine")
        .unwrap_or(true);
    if is_in_engine {
        quote!(grafiek_engine)
    } else {
        quote!(crate)
    }
}

#[proc_macro_derive(EnumSchema)]
pub fn derive_schema_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Enum(data) => {
            let variants: Vec<_> = data.variants.iter().collect();
            let name = &input.ident;
            let krate = crate_path();

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
                impl #krate::traits::SchemaEnum for #name {
                    const VARIANTS : &'static [(&str, i32)] = &[
                        #(
                            (stringify!(#variant_names), #name::#variant_names as i32),
                        )*
                    ];
                }

                impl #krate::Extract for #name {
                   fn extract(value: #krate::ValueRef<'_>) -> std::result::Result<Self, #krate::ValueError> {
                       match value {
                           #krate::ValueRef::I32(v) => {
                               match *v {
                                   #(
                                    i if #name::#variant_names as i32 == i => {
                                        Ok(#name::#variant_names)
                                    },
                                   )*
                                   _ => Err(#krate::ValueError::InvalidEnum)
                               }
                           },
                           other => Err(#krate::ValueError::TypeMismatch {
                               wanted: "i32".to_string(),
                               found: format!("{:?}", other),
                           }),
                       }
                   }
                }
            }
            .into()
        }
        _ => syn::Error::new_spanned(&input, "SchemaEnum can only be derived for enums.")
            .to_compile_error()
            .into(),
    }
}

#[proc_macro_derive(InputSchema, attributes(meta, label))]
pub fn derive_input_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(OutputSchema, attributes(meta, label))]
pub fn derive_output_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_schema_impl(input, SchemaKind::Output) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(ConfigSchema, attributes(meta, label, on_node_body))]
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

/// Parsed attributes from a field
struct SlotAttrs {
    label: String,
    /// Initializer for extra metadata, straight rust copy and pasted
    meta: Option<TokenStream2>,
    on_node_body: bool,
}

impl SlotAttrs {
    fn parse(field: &Field, default_label: &str) -> syn::Result<Self> {
        let label = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("label"))
            .map(|attr| attr.parse_args::<syn::LitStr>().map(|lit| lit.value()))
            .transpose()?
            .unwrap_or_else(|| default_label.to_string());

        let meta = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("meta"))
            .map(|m| m.meta.require_list().map(|c| c.tokens.clone()))
            .transpose()?;

        let on_node_body = field
            .attrs
            .iter()
            .any(|a| a.path().is_ident("on_node_body"));

        Ok(Self {
            label,
            meta,
            on_node_body,
        })
    }

    fn generate(&self, field_type: &syn::Type, krate: &TokenStream2) -> TokenStream2 {
        let label = &self.label;

        let base = if let Some(meta_tokens) = &self.meta {
            quote! {
                #krate::SlotDef::with_metadata(
                    <#field_type as #krate::AsValueType>::VALUE_TYPE,
                    #label,
                    #krate::ExtendedMetadata::from(#meta_tokens),
                )
            }
        } else {
            quote! {
                {
                    let value_type = <#field_type as #krate::AsValueType>::VALUE_TYPE;
                    if let Some(extended) = <#field_type as #krate::AsValueType>::default_metadata() {
                        #krate::SlotDef::with_metadata(value_type, #label, extended)
                    } else {
                        #krate::SlotDef::new(value_type, #label)
                    }
                }
            }
        };

        let setters = [
            self.on_node_body
                .then(|| quote! { slot.set_on_node_body(true); }),
            // add more setters here
        ]
        .into_iter()
        .flatten();

        quote! {
            {
                #[allow(unused_mut)]
                let mut slot = #base;
                #(#setters)*
                slot
            }
        }
    }
}

/// Generate the SlotDef expression for a field
fn generate_slot_def(
    field: &Field,
    field_name_str: &str,
    krate: &TokenStream2,
) -> syn::Result<TokenStream2> {
    let attrs = SlotAttrs::parse(field, field_name_str)?;
    Ok(attrs.generate(&field.ty, krate))
}

fn derive_schema_impl(input: DeriveInput, kind: SchemaKind) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let krate = crate_path();

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
        .map(|(field, name_str)| generate_slot_def(field, name_str, &krate))
        .collect::<syn::Result<_>>()?;

    let field_indices: Vec<_> = (0..field_names.len()).collect();

    let schema_impl = quote! {
        impl #krate::traits::Schema for #name {
            fn fields() -> Vec<#krate::SlotDef> {
                vec![ #( #slot_defs, )* ]
            }

            fn try_extract(values: #krate::Config) -> #krate::error::Result<Self> {
                use #krate::InputsExt;
                Ok(Self {
                    #(
                        #field_names: values.extract(#field_indices)?,
                    )*
                })
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
            impl #krate::traits::InputSchema for #name {}
        },
        SchemaKind::Output => quote! {
            impl #krate::traits::OutputSchema for #name {
                fn try_write(&self, mut outputs: #krate::Outputs) -> #krate::error::Result<()> {
                    todo!("try_write not yet implemented")
                }
            }
        },
        SchemaKind::Config => quote! {
            impl #krate::traits::ConfigSchema for #name {}
        },
    };

    Ok(quote! {
        use #krate::traits::Schema as _;
        #schema_impl
        #default_impl
        #kind_impl
    }
    .into())
}
