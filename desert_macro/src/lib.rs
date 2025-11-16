use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Expr, Fields, Lit, LitStr, Meta, Token, Type};

#[derive(Debug, Clone)]
struct DesertAttributes {
    transparent: bool,
    sorted_constructors: bool,
    custom: Option<Type>,
    evolution_steps: Vec<proc_macro2::TokenStream>,
    field_defaults: HashMap<String, Expr>,
}

fn parse_desert_attributes(attrs: &[Attribute]) -> DesertAttributes {
    let mut transparent = false;
    let mut sorted_constructors = false;
    let mut custom: Option<Type> = None;
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
                        if name_value.path.is_ident("custom") {
                            let ty: Type = syn::parse2(name_value.value.to_token_stream())
                                .expect("custom attribute must be a type");
                            custom = Some(ty);
                        } else {
                            panic!(
                                "Invalid desert attribute: {:?}",
                                name_value.path.get_ident()
                            );
                        }
                    }
                }
            }
        }
    }
    DesertAttributes {
        transparent,
        sorted_constructors,
        custom,
        evolution_steps,
        field_defaults,
    }
}

fn check_raw(ident: &Ident) -> (String, bool) {
    let ident_s = ident.to_string();
    if let Some(raw_ident_s) = ident_s.strip_prefix("r#") {
        (raw_ident_s.to_string(), true)
    } else {
        (ident_s, false)
    }
}

fn get_metadata_ident(name: &Ident) -> Ident {
    let (base_name, raw) = check_raw(name);
    if raw {
        Ident::new_raw(
            &format!("{base_name}_metadata").to_uppercase(),
            Span::call_site(),
        )
    } else {
        Ident::new(
            &format!("{base_name}_metadata").to_uppercase(),
            Span::call_site(),
        )
    }
}

fn get_case_metadata_ident(name: &Ident, case_name: &Ident) -> Ident {
    let (base_name, raw1) = check_raw(name);
    let (base_case_name, raw2) = check_raw(case_name);
    if raw1 || raw2 {
        Ident::new_raw(
            &format!("{base_name}_{base_case_name}_metadata").to_uppercase(),
            Span::call_site(),
        )
    } else {
        Ident::new(
            &format!("{base_name}_{base_case_name}_metadata").to_uppercase(),
            Span::call_site(),
        )
    }
}

// TODO: attribute to force/disable option field detection for a field (because it's based on names only)
// TODO: attribute to use different field names (for Scala compatibility)
#[proc_macro_derive(BinaryCodec, attributes(desert, transient))]
pub fn derive_binary_codec(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("derive input");

    let attrs = parse_desert_attributes(&ast.attrs);
    let transparent = attrs.transparent;
    let use_sorted_constructors = attrs.sorted_constructors;
    let evolution_steps = &attrs.evolution_steps;
    let field_defaults = attrs.field_defaults;
    let version = evolution_steps.len();
    let vplus1 = version + 1;

    let mut push_evolution_steps = Vec::new();
    for evolution_step in evolution_steps {
        push_evolution_steps.push(quote! {
            evolution_steps.push(#evolution_step);
        });
    }

    let name = &ast.ident;
    let metadata_name = get_metadata_ident(name);

    // Process generics for the impl blocks
    let mut generics_for_impl = ast.generics.clone();
    let mut where_clauses = Vec::new();
    for param in &generics_for_impl.params {
        if let syn::GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            where_clauses.push(format!(
                "{}: desert_rust::BinarySerializer + desert_rust::BinaryDeserializer",
                ident
            ));
        }
    }
    if !where_clauses.is_empty() {
        let where_clause_str = format!("where {}", where_clauses.join(", "));
        generics_for_impl.where_clause = Some(syn::parse_str(&where_clause_str).unwrap());
    }

    let (impl_generics, ty_generics, where_clause) = generics_for_impl.split_for_impl();

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
                            impl #impl_generics desert_rust::BinarySerializer for #name #ty_generics #where_clause {
                                fn serialize<Output: desert_rust::BinaryOutput>(&self, context: &mut desert_rust::SerializationContext<Output>) -> desert_rust::Result<()> {
                                    desert_rust::BinarySerializer::serialize(&self.0, context)
                                }
                            }
                            impl #impl_generics desert_rust::BinaryDeserializer for #name #ty_generics #where_clause {
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
                            impl #impl_generics desert_rust::BinarySerializer for #name #ty_generics #where_clause {
                                fn serialize<Output: desert_rust::BinaryOutput>(&self, context: &mut desert_rust::SerializationContext<Output>) -> desert_rust::Result<()> {
                                    desert_rust::BinarySerializer::serialize(&self.#field_name, context)
                                }
                            }
                            impl #impl_generics desert_rust::BinaryDeserializer for #name #ty_generics #where_clause {
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

            let mut ser_cases = Vec::new();
            let mut deser_cases = Vec::new();

            let mut variants = enum_data.variants.iter().cloned().collect::<Vec<_>>();
            if use_sorted_constructors {
                variants.sort_by_key(|variant| variant.ident.to_string());
            }

            let mut effective_case_idx: u32 = 0;
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
                    let variant_attrs = parse_desert_attributes(&variant.attrs);
                    let variant_transparent = variant_attrs.transparent;
                    let variant_custom = variant_attrs.custom;
                    let case_evolution_steps = &variant_attrs.evolution_steps;
                    let case_field_defaults = variant_attrs.field_defaults;
                    if variant_transparent || variant_custom.is_some() {
                        // transparent on a variant case means we don't want to treat it as an evolvable record type, but directly
                        // serialize the single-element field of it as-is
                        // custom is similar to transparent but wraps the field in the provided type
                        if variant.fields.len() != 1 {
                            panic!("transparent/custom not allowed on non-single field variants");
                        } else {
                            let single_field = match &variant.fields {
                                Fields::Named(named_fields) => {
                                    named_fields.named[0].ident.clone().unwrap()
                                }
                                Fields::Unnamed(_) => Ident::new("field0", Span::call_site()),
                                Fields::Unit => unreachable!(),
                            };

                            if let Some(ref custom_type) = variant_custom {
                                ser_cases.push(
                                    quote! {
                                        #pattern => {
                                            serializer.write_constructor(
                                                #effective_case_idx,
                                                |desert_inner_serialization_context| {
                                                    let wrapped = #custom_type(::std::borrow::Cow::Borrowed(#single_field));
                                                    desert_rust::BinarySerializer::serialize(&wrapped, desert_inner_serialization_context)
                                                }
                                            )?;
                                        }
                                    }
                                );
                            } else {
                                ser_cases.push(
                                    quote! {
                                        #pattern => {
                                            serializer.write_constructor(
                                                #effective_case_idx,
                                                |desert_inner_serialization_context| {
                                                    desert_rust::BinarySerializer::serialize(#single_field, desert_inner_serialization_context)
                                                }
                                            )?;
                                        }
                                    }
                                );
                            }

                            let construct_case = if let Some(ref custom_type) = variant_custom {
                                match &variant.fields {
                                    Fields::Unit => unreachable!(),
                                    Fields::Named(_) => {
                                        quote! { #name::#case_name {
                                                #single_field: {
                                                    let #custom_type(inner) = desert_rust::BinaryDeserializer::deserialize(context)?;
                                                    inner.into_owned()
                                                }
                                            }
                                        }
                                    }
                                    Fields::Unnamed(_) => {
                                        quote! { #name::#case_name({
                                                let #custom_type(inner) = desert_rust::BinaryDeserializer::deserialize(context)?;
                                                inner.into_owned()
                                            })
                                        }
                                    }
                                }
                            } else {
                                match &variant.fields {
                                    Fields::Unit => unreachable!(),
                                    Fields::Named(_) => {
                                        quote! { #name::#case_name {
                                                #single_field: desert_rust::BinaryDeserializer::deserialize(context)?
                                            }
                                        }
                                    }
                                    Fields::Unnamed(_) => {
                                        quote! { #name::#case_name(desert_rust::BinaryDeserializer::deserialize(context)?) }
                                    }
                                }
                            };

                            deser_cases.push(quote! {
                                #effective_case_idx => {
                                    Ok(#construct_case)
                                },
                            });
                        }
                    } else {
                        let version = case_evolution_steps.len();
                        let vplus1 = version + 1;

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

                        let case_metadata_name = get_case_metadata_ident(name, case_name);

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

                        ser_cases.push(
                            quote! {
                            #pattern => {
                                serializer.write_constructor(
                                    #effective_case_idx,
                                    |desert_inner_serialization_context| {
                                        let mut serializer = desert_rust::adt::AdtSerializer::<_, #vplus1>::#new_v(&#case_metadata_name, desert_inner_serialization_context);
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

                        deser_cases.push(
                            quote! {
                                #effective_case_idx => {
                                    let stored_version = context.read_u8()?;
                                    let mut deserializer = if stored_version == 0 {
                                        desert_rust::adt::AdtDeserializer::<#vplus1>::new_v0(&#case_metadata_name, context)?
                                    } else {
                                        desert_rust::adt::AdtDeserializer::<#vplus1>::new(&#case_metadata_name, context, stored_version)?
                                    };
                                    Ok(#construct_case)
                                },
                            }
                        );
                    }

                    effective_case_idx += 1;
                } else {
                    let name_string = name.to_string();
                    let case_name_string = case_name.to_string();
                    ser_cases.push(quote! {
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
                    #(#ser_cases),*
                }
            });

            deserialization_commands.push(quote! {
                 let desert_constructor_idx = deserializer.read_constructor_idx()?;
                 match desert_constructor_idx {
                     #(#deser_cases)*
                    other => {
                        Err(desert_rust::Error::InvalidConstructorId {
                            type_name: stringify!(#name).to_string(),
                            constructor_id: other,
                        })
                    }
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
        }
    };

    let gen = quote! {
        #(#metadata)*

        #[allow(unused_variables)]
        impl #impl_generics desert_rust::BinarySerializer for #name #ty_generics #where_clause {
            fn serialize<Output: desert_rust::BinaryOutput>(&self, context: &mut desert_rust::SerializationContext<Output>) -> desert_rust::Result<()> {
                let mut serializer = desert_rust::adt::AdtSerializer::<_, #vplus1>::#new_v(&#metadata_name, context);
                #(#serialization_commands)*
                serializer.finish()
            }
        }

        impl #impl_generics desert_rust::BinaryDeserializer for #name #ty_generics #where_clause {
            fn deserialize<'a, 'b>(context: &'a mut desert_rust::DeserializationContext<'b>) -> desert_rust::Result<Self> {
                use desert_rust::BinaryInput;

                let stored_version = context.read_u8()?;
                let mut deserializer = if stored_version == 0 {
                    desert_rust::adt::AdtDeserializer::<#vplus1>::new_v0(&#metadata_name, context)?
                } else {
                    desert_rust::adt::AdtDeserializer::<#vplus1>::new(&#metadata_name, context, stored_version)?
                };
                #deserialization
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
