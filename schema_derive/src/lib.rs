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

                impl From<#name> for #krate::Value {
                    fn from(v: #name) -> Self {
                        #krate::Value::I32(v as i32)
                    }
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

#[proc_macro_derive(
    ConfigSchema,
    attributes(meta, label, on_node_body, noninteractive, default)
)]
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
    default: Option<TokenStream2>,
    on_node_body: bool,
    noninteractive: bool,
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

        let default = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("default"))
            .map(|m| m.meta.require_list().map(|c| c.tokens.clone()))
            .transpose()?;

        let on_node_body = field
            .attrs
            .iter()
            .any(|a| a.path().is_ident("on_node_body"));

        let noninteractive = field
            .attrs
            .iter()
            .any(|a| a.path().is_ident("noninteractive"));

        Ok(Self {
            label,
            meta,
            default,
            on_node_body,
            noninteractive,
        })
    }

    fn generate_builder_call(
        &self,
        field_type: &syn::Type,
        add_method: &TokenStream2,
    ) -> TokenStream2 {
        let label = &self.label;

        let meta_call = self.meta.as_ref().map(|m| quote! { .meta(#m) });
        let default_call = self.default.as_ref().map(|d| quote! { .default(#d) });
        let on_node_body_call = self
            .on_node_body
            .then(|| quote! { .show_on_node_body(true) });
        let interactive_call = self.noninteractive.then(|| quote! { .interactive(false) });

        quote! {
            registry.#add_method::<#field_type>(#label)
                #meta_call
                #default_call
                #on_node_body_call
                #interactive_call
                .build();
        }
    }
}

fn generate_register_call(
    field: &Field,
    field_name_str: &str,
    add_method: &TokenStream2,
) -> syn::Result<TokenStream2> {
    let attrs = SlotAttrs::parse(field, field_name_str)?;
    Ok(attrs.generate_builder_call(&field.ty, add_method))
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

    let add_method = match kind {
        SchemaKind::Input => quote! { add_input },
        SchemaKind::Output => quote! { add_output },
        SchemaKind::Config => quote! { add_config },
    };

    let register_calls: Vec<TokenStream2> = fields
        .iter()
        .zip(field_name_strs.iter())
        .map(|(field, name_str)| generate_register_call(field, name_str, &add_method))
        .collect::<syn::Result<_>>()?;

    let field_indices: Vec<_> = (0..field_names.len()).collect();

    let schema_impl = quote! {
        impl #krate::traits::Schema for #name {
            fn register(registry: &mut #krate::SignatureRegistery) {
                #( #register_calls )*
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
