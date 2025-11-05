use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Expr, Fields, Lit, LitStr, Meta, Token, Type};

fn parse_desert_attributes(
    attrs: &[Attribute],
) -> (
    bool,
    bool,
    Vec<proc_macro2::TokenStream>,
    HashMap<String, Expr>,
) {
    let mut transparent = false;
    let mut sorted_constructors = false;
    let mut evolution_steps = Vec::new();
    let mut field_defaults = HashMap::new();

    for attr in attrs {
        if attr.path().is_ident("desert") {
            let nested = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .expect("desert args");
            for meta in nested {
                match meta {
                    Meta::Path(path) => {
                        if path.is_ident("transparent") {
                            transparent = true;
                        } else if path.is_ident("sorted_constructors") {
                            sorted_constructors = true;
                        } else {
                            panic!("Unknown desert attribute: {:?}", path.get_ident());
                        }
                    }
                    Meta::List(list) => {
                        if list.path.is_ident("evolution") {
                            let nested_evolution = list
                                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                                .expect("evolution steps");
                            for meta in nested_evolution {
                                match meta {
                                    Meta::Path(path) => {
                                        panic!("Invalid evolution step: {:?}", path.get_ident());
                                    }
                                    Meta::List(list) => {
                                        if list.path.is_ident("FieldAdded") {
                                            let args = list
                                                .parse_args_with(
                                                    Punctuated::<Expr, Token![,]>::parse_terminated,
                                                )
                                                .expect("FieldAdded arguments");
                                            if args.len() != 2 {
                                                panic!(
                                                    "Invalid number of arguments for FieldAdded"
                                                );
                                            }
                                            let field_name = match &args[0] {
                                                Expr::Lit(lit) => match &lit.lit {
                                                    Lit::Str(field_name) => field_name.value(),
                                                    _ => panic!("Invalid field name for FieldAdded - must be a string literal"),
                                                }
                                                other => panic!("Invalid field name for FieldAdded - must be a string literal but it was {}",
                                                                expr_type(other)
                                                ),
                                            };
                                            let field_default = &args[1];

                                            field_defaults
                                                .insert(field_name.clone(), field_default.clone());
                                            evolution_steps.push(quote! {
                                                desert_rust::Evolution::FieldAdded {
                                                    name: #field_name.to_string(),
                                                }
                                            });
                                        } else if list.path.is_ident("FieldMadeOptional") {
                                            let field_name_lit: LitStr = list
                                                .parse_args()
                                                .expect("FieldMadeOptional argument");
                                            let field_name = field_name_lit.value();

                                            evolution_steps.push(quote! {
                                                desert_rust::Evolution::FieldMadeOptional {
                                                    name: #field_name.to_string(),
                                                }
                                            });
                                        } else if list.path.is_ident("FieldRemoved") {
                                            let field_name_lit: LitStr = list
                                                .parse_args()
                                                .expect("FieldMadeOptional argument");
                                            let field_name = field_name_lit.value();

                                            evolution_steps.push(quote! {
                                                desert_rust::Evolution::FieldRemoved {
                                                    name: #field_name.to_string(),
                                                }
                                            });
                                        } else if list.path.is_ident("FieldMadeTransient") {
                                            let field_name_lit: LitStr = list
                                                .parse_args()
                                                .expect("FieldMadeOptional argument");
                                            let field_name = field_name_lit.value();

                                            evolution_steps.push(quote! {
                                                desert_rust::Evolution::FieldMadeTransient {
                                                    name: #field_name.to_string(),
                                                }
                                            });
                                        } else {
                                            panic!(
                                                "Invalid evolution step: {:?}",
                                                list.path.get_ident()
                                            );
                                        }
                                    }
                                    Meta::NameValue(name_value) => {
                                        panic!(
                                            "Invalid evolution step: {:?}",
                                            name_value.path.get_ident()
                                        );
                                    }
                                }
                            }
                        } else {
                            panic!("Unknown desert list attribute: {:?}", list.path.get_ident());
                        }
                    }
                    Meta::NameValue(name_value) => {
                        panic!(
                            "Invalid desert attribute: {:?}",
                            name_value.path.get_ident()
                        );
                    }
                }
            }
        }
    }
    (
        transparent,
        sorted_constructors,
        evolution_steps,
        field_defaults,
    )
}

// TODO: attribute to force/disable option field detection for a field (because it's based on names only)
// TODO: attribute to use different field names (for Scala compatibility)
#[proc_macro_derive(BinaryCodec, attributes(desert, transient))]
pub fn derive_binary_codec(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("derive input");

    let (transparent, use_sorted_constructors, evolution_steps, field_defaults) =
        parse_desert_attributes(&ast.attrs);
    let version = evolution_steps.len();
    let mut push_evolution_steps = Vec::new();
    for evolution_step in evolution_steps {
        push_evolution_steps.push(quote! {
            evolution_steps.push(#evolution_step);
        });
    }

    let name = &ast.ident;
    let metadata_name = Ident::new(
        &format!("{name}_metadata").to_uppercase(),
        Span::call_site(),
    );

    let mut metadata = Vec::new();
    let mut serialization_commands = Vec::new();
    let mut deserialization_commands = Vec::new();
    let is_record;

    match ast.data {
        Data::Struct(struct_data) => {
            if transparent {
                match &struct_data.fields {
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        let gen = quote! {
                            impl desert_rust::BinarySerializer for #name {
                                fn serialize<Output: desert_rust::BinaryOutput>(&self, context: &mut desert_rust::SerializationContext<Output>) -> desert_rust::Result<()> {
                                    desert_rust::BinarySerializer::serialize(&self.0, context)
                                }
                            }
                            impl desert_rust::BinaryDeserializer for #name {
                                fn deserialize<'a, 'b>(context: &'a mut desert_rust::DeserializationContext<'b>) -> desert_rust::Result<Self> {
                                    Ok(Self(desert_rust::BinaryDeserializer::deserialize(context)?))
                                }
                            }
                        };
                        return gen.into();
                    }
                    Fields::Named(fields) if fields.named.len() == 1 => {
                        let field_name = &fields.named[0].ident;
                        let gen = quote! {
                            impl desert_rust::BinarySerializer for #name {
                                fn serialize<Output: desert_rust::BinaryOutput>(&self, context: &mut desert_rust::SerializationContext<Output>) -> desert_rust::Result<()> {
                                    desert_rust::BinarySerializer::serialize(&self.#field_name, context)
                                }
                            }
                            impl desert_rust::BinaryDeserializer for #name {
                                fn deserialize<'a, 'b>(context: &'a mut desert_rust::DeserializationContext<'b>) -> desert_rust::Result<Self> {
                                    Ok(Self { #field_name: desert_rust::BinaryDeserializer::deserialize(context)? })
                                }
                            }
                        };
                        return gen.into();
                    }
                    _ => panic!("transparent can only be used on single-element tuple structs"),
                }
            }
            is_record = true;
            let mut field_patterns = Vec::new();
            for field in struct_data.fields.iter() {
                let field_ident = field.ident.as_ref().unwrap();
                field_patterns.push(quote! { #field_ident });
            }
            serialization_commands.push(quote! {
               let #name { #(#field_patterns),* } = self;
            });
            derive_field_serialization(
                field_defaults,
                &mut serialization_commands,
                &mut deserialization_commands,
                &struct_data.fields,
            );
        }
        Data::Enum(enum_data) => {
            if transparent {
                panic!("transparent is not supported for enums");
            }
            is_record = false;

            let mut cases = Vec::new();

            let mut variants = enum_data.variants.iter().cloned().collect::<Vec<_>>();
            if use_sorted_constructors {
                variants.sort_by_key(|variant| variant.ident.to_string());
            }

            let mut effective_case_idx = 0;
            for variant in variants {
                let is_transient = variant
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("transient"));
                let case_name = &variant.ident;

                let pattern = match &variant.fields {
                    Fields::Unit => {
                        quote! { #name::#case_name }
                    }
                    Fields::Named(named_fields) => {
                        let mut field_patterns = Vec::new();
                        for field in named_fields.named.iter() {
                            let field_ident = field.ident.as_ref().unwrap();
                            field_patterns.push(quote! { #field_ident });
                        }
                        quote! { #name::#case_name { #(#field_patterns),* } }
                    }
                    Fields::Unnamed(unnamed_fields) => {
                        let mut field_patterns = Vec::new();
                        for n in 0..unnamed_fields.unnamed.len() {
                            let field_ident = Ident::new(&format!("field{}", n), Span::call_site());
                            field_patterns.push(quote! { #field_ident });
                        }
                        quote! { #name::#case_name(#(#field_patterns),*) }
                    }
                };

                if !is_transient {
                    let (variant_transparent, _, case_evolution_steps, case_field_defaults) =
                        parse_desert_attributes(&variant.attrs);
                    if variant_transparent {
                        panic!("transparent not allowed on variants");
                    }
                    let version = case_evolution_steps.len();
                    let mut case_push_evolution_steps = Vec::new();
                    for evolution_step in case_evolution_steps {
                        case_push_evolution_steps.push(quote! {
                            evolution_steps.push(#evolution_step);
                        });
                    }
                    let mut case_serialization_commands = Vec::new();
                    let mut case_deserialization_commands = Vec::new();

                    let new_v = if version == 0 {
                        quote! { new_v0 }
                    } else {
                        quote! { new }
                    };

                    let case_metadata_name = Ident::new(
                        &format!("{name}_{case_name}_metadata").to_uppercase(),
                        Span::call_site(),
                    );

                    metadata.push(quote! {
                        desert_rust::lazy_static! {
                            static ref #case_metadata_name: desert_rust::adt::AdtMetadata = {
                                let mut evolution_steps: Vec<desert_rust::Evolution> = Vec::new();
                                evolution_steps.push(desert_rust::Evolution::InitialVersion);
                                #(#case_push_evolution_steps)*

                                desert_rust::adt::AdtMetadata::new(
                                    evolution_steps,
                                )
                            };
                        }
                    });

                    derive_field_serialization(
                        case_field_defaults,
                        &mut case_serialization_commands,
                        &mut case_deserialization_commands,
                        &variant.fields,
                    );

                    cases.push(
                        quote! {
                        #pattern => {
                            serializer.write_constructor(
                                #effective_case_idx as u32,
                                |context| {
                                    let mut serializer = desert_rust::adt::AdtSerializer::#new_v(&#case_metadata_name, context);
                                    #(#case_serialization_commands)*
                                    serializer.finish()
                                }
                            )?;
                        }
                    }
                    );

                    let construct_case = match &variant.fields {
                        Fields::Unit => {
                            quote! { #name::#case_name }
                        }
                        Fields::Named(_) => {
                            quote! { #name::#case_name {
                                    #(#case_deserialization_commands)*
                                }
                            }
                        }
                        Fields::Unnamed(_) => {
                            quote! { #name::#case_name(#(#case_deserialization_commands)*) }
                        }
                    };

                    deserialization_commands.push(
                        quote! {
                            if let Some(result) = deserializer.read_constructor(#effective_case_idx as u32,
                                |context| {
                                    let stored_version = context.read_u8()?;
                                    if stored_version == 0 {
                                        let mut deserializer = desert_rust::adt::AdtDeserializer::new_v0(&#case_metadata_name, context)?;
                                        Ok(#construct_case)
                                    } else {
                                        let mut deserializer = desert_rust::adt::AdtDeserializer::new(&#case_metadata_name, context, stored_version)?;
                                        Ok(#construct_case)
                                    }
                                }
                            )? {
                                return Ok(result)
                            }
                       }
                    );

                    effective_case_idx += 1;
                } else {
                    let name_string = name.to_string();
                    let case_name_string = case_name.to_string();
                    cases.push(quote! {
                        #pattern => {
                            return Err(desert_rust::Error::SerializingTransientConstructor {
                                type_name: #name_string.to_string(),
                                constructor_name: #case_name_string.to_string(),
                            });
                        }
                    });
                }
            }

            serialization_commands.push(quote! {
                match self {
                    #(#cases),*
                }
            });
        }
        Data::Union(_) => {
            panic!("Unions are not supported");
        }
    }

    metadata.push(quote! {
        desert_rust::lazy_static! {
            static ref #metadata_name: desert_rust::adt::AdtMetadata = {
                let mut evolution_steps: Vec<desert_rust::Evolution> = Vec::new();
                evolution_steps.push(desert_rust::Evolution::InitialVersion);
                #(#push_evolution_steps)*

                desert_rust::adt::AdtMetadata::new(
                    evolution_steps,
                )
            };
        }
    });

    let new_v = if version == 0 {
        quote! { new_v0 }
    } else {
        quote! { new }
    };

    let deserialization = if is_record {
        quote! {
            Ok(Self {
                    #(#deserialization_commands)*
            })
        }
    } else {
        quote! {
            #(#deserialization_commands)*
            Err(desert_rust::Error::InvalidConstructorId {
                type_name: stringify!(#name).to_string(),
                constructor_id: deserializer.read_or_get_constructor_idx().unwrap_or(u32::MAX),
            })
        }
    };

    let gen = quote! {
        #(#metadata)*

        #[allow(unused_variables)]
        impl desert_rust::BinarySerializer for #name {
            fn serialize<Output: desert_rust::BinaryOutput>(&self, context: &mut desert_rust::SerializationContext<Output>) -> desert_rust::Result<()> {
                let mut serializer = desert_rust::adt::AdtSerializer::#new_v(&#metadata_name, context);
                #(#serialization_commands)*
                serializer.finish()
            }
        }

        impl desert_rust::BinaryDeserializer for #name {
            fn deserialize<'a, 'b>(context: &'a mut desert_rust::DeserializationContext<'b>) -> desert_rust::Result<Self> {
                use desert_rust::BinaryInput;

                let stored_version = context.read_u8()?;
                if stored_version == 0 {
                    let mut deserializer = desert_rust::adt::AdtDeserializer::new_v0(&#metadata_name, context)?;
                    #deserialization
                } else {
                    let mut deserializer = desert_rust::adt::AdtDeserializer::new(&#metadata_name, context, stored_version)?;
                    #deserialization
                }
            }
        }
    };

    gen.into()
}

fn derive_field_serialization(
    field_defaults: HashMap<String, Expr>,
    serialization_commands: &mut Vec<proc_macro2::TokenStream>,
    deserialization_commands: &mut Vec<proc_macro2::TokenStream>,
    fields: &Fields,
) {
    for (n, field) in fields.iter().enumerate() {
        let n_ident = Ident::new(&format!("field{n}"), Span::call_site());
        let field_ident = field.ident.as_ref().unwrap_or(&n_ident);
        let field_name = field_ident.to_string();

        let mut transient = None;
        for attr in &field.attrs {
            if attr.path().is_ident("transient") {
                let args = attr
                    .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                    .unwrap_or_default();
                if args.len() != 1 {
                    panic!("#[transient(default)] on fields needs a default value");
                }
                let field_default = args[0].clone();
                transient = Some(field_default);
            }
        }

        match transient {
            None => {
                serialization_commands.push(quote! {
                    serializer.write_field(#field_name, &#field_ident)?;
                });

                if is_option(&field.ty) {
                    match field_defaults.get(&field_name) {
                        Some(field_default) => {
                            if field.ident.is_some() {
                                deserialization_commands.push(quote! {
                                    #field_ident: deserializer.read_optional_field(#field_name, Some(#field_default))?,
                                });
                            } else {
                                deserialization_commands.push(quote! {
                                    deserializer.read_optional_field(#field_name, Some(#field_default))?,
                                });
                            }
                        }
                        None => {
                            if field.ident.is_some() {
                                deserialization_commands.push(quote! {
                                   #field_ident: deserializer.read_optional_field(#field_name, None)?,
                                });
                            } else {
                                deserialization_commands.push(quote! {
                                   deserializer.read_optional_field(#field_name, None)?,
                                });
                            }
                        }
                    }
                } else {
                    match field_defaults.get(&field_name) {
                        Some(field_default) => {
                            if field.ident.is_some() {
                                deserialization_commands.push(quote! {
                                    #field_ident: deserializer.read_field(#field_name, Some(#field_default))?,
                                });
                            } else {
                                deserialization_commands.push(quote! {
                                    deserializer.read_field(#field_name, Some(#field_default))?,
                                });
                            }
                        }
                        None => {
                            if field.ident.is_some() {
                                deserialization_commands.push(quote! {
                                   #field_ident: deserializer.read_field(#field_name, None)?,
                                });
                            } else {
                                deserialization_commands.push(quote! {
                                   deserializer.read_field(#field_name, None)?,
                                });
                            }
                        }
                    }
                }
            }
            Some(transient_default_value) => {
                if field.ident.is_some() {
                    deserialization_commands.push(quote! {
                      #field_ident: #transient_default_value,
                    });
                } else {
                    deserialization_commands.push(quote! {
                      #transient_default_value,
                    });
                }
            }
        }
    }
}

fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Group(group) => is_option(&group.elem),
        Type::Paren(paren) => is_option(&paren.elem),
        Type::Path(type_path) => {
            if type_path.qself.is_none() {
                let idents = type_path
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>();
                idents == vec!["Option"]
                    || idents == vec!["std", "option", "Option"]
                    || idents == vec!["core", "option", "Option"]
            } else {
                false
            }
        }
        _ => false,
    }
}

fn expr_type(expr: &Expr) -> String {
    match expr {
        Expr::Array(_) => "array".to_string(),
        Expr::Assign(_) => "assign".to_string(),
        Expr::Async(_) => "async".to_string(),
        Expr::Await(_) => "await".to_string(),
        Expr::Binary(_) => "binary".to_string(),
        Expr::Block(_) => "block".to_string(),
        Expr::Break(_) => "break".to_string(),
        Expr::Call(_) => "call".to_string(),
        Expr::Cast(_) => "cast".to_string(),
        Expr::Closure(_) => "closure".to_string(),
        Expr::Const(_) => "const".to_string(),
        Expr::Continue(_) => "continue".to_string(),
        Expr::Field(_) => "field".to_string(),
        Expr::ForLoop(_) => "for loop".to_string(),
        Expr::Group(_) => "group".to_string(),
        Expr::If(_) => "if".to_string(),
        Expr::Index(_) => "index".to_string(),
        Expr::Infer(_) => "infer".to_string(),
        Expr::Let(_) => "let".to_string(),
        Expr::Lit(_) => "lit".to_string(),
        Expr::Loop(_) => "loop".to_string(),
        Expr::Macro(_) => "macro".to_string(),
        Expr::Match(_) => "match".to_string(),
        Expr::MethodCall(_) => "method call".to_string(),
        Expr::Paren(_) => "paren".to_string(),
        Expr::Path(_) => "path".to_string(),
        Expr::Range(_) => "range".to_string(),
        Expr::Reference(_) => "reference".to_string(),
        Expr::Repeat(_) => "repeat".to_string(),
        Expr::Return(_) => "return".to_string(),
        Expr::Struct(_) => "struct".to_string(),
        Expr::Try(_) => "try".to_string(),
        Expr::TryBlock(_) => "try block".to_string(),
        Expr::Tuple(_) => "tuple".to_string(),
        Expr::Unary(_) => "unary".to_string(),
        Expr::Unsafe(_) => "unsafe".to_string(),
        Expr::Verbatim(_) => "verbatim".to_string(),
        Expr::While(_) => "while".to_string(),
        Expr::Yield(_) => "yield".to_string(),
        _ => "unknown".to_string(),
    }
}
