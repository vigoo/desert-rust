use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Expr, Lit, LitStr, Meta, Token, Type};

fn evolution_steps_from_attributes(
    attrs: Vec<Attribute>,
) -> (Vec<proc_macro2::TokenStream>, HashMap<String, Expr>) {
    let mut evolution_steps = Vec::new();
    let mut field_defaults = HashMap::new();
    for attr in attrs {
        if attr.path().is_ident("evolution") {
            let nested = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .expect("evolution steps");
            for meta in nested {
                match meta {
                    Meta::Path(path) => {
                        panic!("Invalid evolution step: {:?}", path.get_ident());
                    }
                    Meta::List(list) => {
                        if list.path.is_ident("FieldAdded") {
                            let args = list
                                .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                                .expect("FieldAdded arguments");
                            if args.len() != 2 {
                                panic!("Invalid number of arguments for FieldAdded");
                            }
                            let field_name = match &args[0] {
                                Expr::Lit(lit) =>
                                    match &lit.lit {
                                        Lit::Str(field_name) => field_name.value(),
                                        _ => panic!("Invalid field name for FieldAdded - must be a string literal"),
                                    }
                                other => panic!("Invalid field name for FieldAdded - must be a string literal but it was {}",
                                                expr_type(other)
                                ),
                            };
                            let field_default = &args[1];

                            println!(
                                "FieldAdded: name = {field_name}, default value: {:?}",
                                quote! { #field_default }
                            );

                            field_defaults.insert(field_name.clone(), field_default.clone());
                            evolution_steps.push(quote! {
                                desert::Evolution::FieldAdded {
                                    name: #field_name.to_string(),
                                }
                            });
                        } else if list.path.is_ident("FieldMadeOptional") {
                            let field_name_lit: LitStr =
                                list.parse_args().expect("FieldMadeOptional argument");
                            let field_name = field_name_lit.value();
                            println!("FieldMadeOptional: name = {field_name}");

                            evolution_steps.push(quote! {
                                desert::Evolution::FieldMadeOptional {
                                    name: #field_name.to_string(),
                                }
                            });
                        } else if list.path.is_ident("FieldRemoved") {
                            let field_name_lit: LitStr =
                                list.parse_args().expect("FieldMadeOptional argument");
                            let field_name = field_name_lit.value();
                            println!("FieldRemoved: name = {field_name}");

                            evolution_steps.push(quote! {
                                desert::Evolution::FieldRemoved {
                                    name: #field_name.to_string(),
                                }
                            });
                        } else if list.path.is_ident("FieldMadeTransient") {
                            let field_name_lit: LitStr =
                                list.parse_args().expect("FieldMadeOptional argument");
                            let field_name = field_name_lit.value();
                            println!("FieldMadeTransient: name = {field_name}");

                            evolution_steps.push(quote! {
                                desert::Evolution::FieldMadeTransient {
                                    name: #field_name.to_string(),
                                }
                            });
                        } else {
                            panic!("Invalid evolution step: {:?}", list.path.get_ident());
                        }
                    }
                    Meta::NameValue(name_value) => {
                        panic!("Invalid evolution step: {:?}", name_value.path.get_ident());
                    }
                }
            }
        }
    }
    (evolution_steps, field_defaults)
}

// TODO: attribute to force/disable option field detection for a field (because it's based on names only)
// TODO: attribute to use different field names (for Scala compatibility)
#[proc_macro_derive(BinaryCodec, attributes(evolution, transient))]
pub fn derive_binary_codec(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("derive input");

    let (evolution_steps, field_defaults) = evolution_steps_from_attributes(ast.attrs);
    let version = evolution_steps.len();
    let mut push_evolution_steps = Vec::new();
    for evolution_step in evolution_steps {
        push_evolution_steps.push(quote! {
            evolution_steps.push(#evolution_step);
        });
    }

    let name = &ast.ident;
    let name_string = name.to_string();
    let metadata_name = Ident::new(
        &format!("{name}_metadata").to_uppercase(),
        Span::call_site(),
    );

    let mut serialization_commands = Vec::new();
    let mut deserialization_commands: Vec<proc_macro2::TokenStream> = Vec::new();

    match ast.data {
        Data::Struct(struct_data) => {
            for field in &struct_data.fields {
                let field_ident = field
                    .ident
                    .as_ref()
                    .expect("Field does not have an identifier");
                let field_name = field_ident.to_string();

                let mut transient = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("transient") {
                        let args = attr
                            .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                            .expect("FieldAdded arguments");
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
                            serializer.write_field(#field_name, &self.#field_ident)?;
                        });

                        if is_option(&field.ty) {
                            match field_defaults.get(&field_name) {
                                Some(field_default) => {
                                    deserialization_commands.push(quote! {
                                        #field_ident: deserializer.read_optional_field(#field_name, Some(#field_default))?,
                                    });
                                }
                                None => {
                                    deserialization_commands.push(quote! {
                                       #field_ident: deserializer.read_optional_field(#field_name, None)?,
                                    });
                                }
                            }
                        } else {
                            match field_defaults.get(&field_name) {
                                Some(field_default) => {
                                    deserialization_commands.push(quote! {
                                        #field_ident: deserializer.read_field(#field_name, Some(#field_default))?,
                                    });
                                }
                                None => {
                                    deserialization_commands.push(quote! {
                                       #field_ident: deserializer.read_field(#field_name, None)?,
                                    });
                                }
                            }
                        }
                    }
                    Some(transient_default_value) => {
                        deserialization_commands.push(quote! {
                          #field_ident: #transient_default_value,
                        });
                    }
                }
            }
        }
        Data::Enum(enum_data) => {
            for variant in &enum_data.variants {

            }
            todo!();
        }
        Data::Union(_) => {
            panic!("Unions are not supported");
        }
    }

    let new_v = if version == 0 {
        quote! { new_v0 }
    } else {
        quote! { new }
    };

    let gen = quote! {
        lazy_static::lazy_static! {
            static ref #metadata_name: desert::adt::AdtMetadata = {
                let mut evolution_steps: Vec<desert::Evolution> = Vec::new();
                evolution_steps.push(desert::Evolution::InitialVersion);
                #(#push_evolution_steps)*

                desert::adt::AdtMetadata::new(
                    #name_string,
                    evolution_steps,
                    &vec![],
                )
            };
        }

        impl desert::BinarySerializer for #name {
            fn serialize<Context: desert::SerializationContext>(&self, context: &mut Context) -> desert::Result<()> {
                let mut serializer = desert::adt::AdtSerializer::#new_v(&#metadata_name, context);
                #(#serialization_commands)*
                serializer.finish()
            }
        }

        impl desert::BinaryDeserializer for #name {
            fn deserialize<Context: desert::DeserializationContext>(context: &mut Context) -> desert::Result<Self> {
                use desert::BinaryInput;

                let stored_version = context.input_mut().read_u8()?;
                if stored_version == 0 {
                    let mut deserializer = desert::adt::AdtDeserializer::new_v0(&#metadata_name, context)?;
                    Ok(Self {
                        #(#deserialization_commands)*
                    })
                } else {
                    let mut deserializer = desert::adt::AdtDeserializer::new(&#metadata_name, context, stored_version)?;
                    Ok(Self {
                        #(#deserialization_commands)*
                    })
                }
            }
        }
    };

    gen.into()
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
