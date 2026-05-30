use quote::ToTokens;
use syn::spanned::Spanned;
use std::collections::HashMap;

/// Creates a `syn::Result::Err` for a particular span with a display message
pub fn spanned_error<T>(span: proc_macro2::Span, message: &str) -> syn::Result<T> {
    syn::Result::Err(syn::Error::new(span, message))
}

fn get_option_inner_type(syntype: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = syntype
        && path.segments.len() == 1
        && let Some(last_segment) = path.segments.last()
        && last_segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(bracket_args) = &last_segment.arguments
        && let syn::AngleBracketedGenericArguments { args, .. } = bracket_args
        && args.len() == 1
        && let Some(bracket_arg) = args.last()
        && let syn::GenericArgument::Type(inner) = bracket_arg
    {
        Some(inner)
    } else {
        None
    }
}

fn get_vec_inner_type(syntype: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = syntype
        && path.segments.len() == 1
        && let Some(last_segment) = path.segments.last()
        && last_segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(bracket_args) = &last_segment.arguments
        && let syn::AngleBracketedGenericArguments { args, .. } = bracket_args
        && args.len() == 1
        && let Some(bracket_arg) = args.last()
        && let syn::GenericArgument::Type(inner) = bracket_arg
    {
        Some(inner)
    } else {
        None
    }
}

fn filter_field_attribute<'a>(
    attr: &'a syn::Attribute,
    attr_name: &str,
) -> Option<&'a syn::Attribute> {
    if let syn::Meta::List(syn::MetaList { path, .. }) = &attr.meta
        && path.segments.len() == 1
        && path.segments.last().is_some()
        && path.segments.last().unwrap().ident.to_string() == attr_name
    {
        Some(attr)
    } else {
        None
    }
}

/// Represents an attribute placed on field 
#[derive(Debug, Clone)]
pub struct KvFieldAttribute {
    #[allow(dead_code)]
    attr: String,
    key: String,
    value: String,
    #[allow(dead_code)]
    span: proc_macro2::Span,
}

impl TryFrom<&syn::Attribute> for KvFieldAttribute {
    type Error = syn::Error;
    
    fn try_from(attribute: &syn::Attribute) -> Result<Self, Self::Error> {
        if let syn::Meta::List(syn::MetaList { path, tokens, .. }) = &attribute.meta {
            if path.segments.len() != 1 {
                let message = "Cannot read field attribute path with multple segments.";
                spanned_error::<String>(attribute.span(), message)?;
            }
            let attr = match path.segments.last() {
                None => {
                    let message = "Attribute must go be placed on named field.";
                    spanned_error(attribute.span(), message)
                },
                Some(attr_name) => {
                    Ok(attr_name.ident.to_string())
                },
            }?;
    
            let symbols: Vec<_> = tokens
                .clone()
                .into_iter()
                .collect();
            if symbols.len() != 3 {
                let message = format!("In field attribute {attr}, unexpected number of symbols: expected 3, found {}", symbols.len());
                spanned_error(tokens.span(), &message)?;
            }
            let key = match &symbols[0] {
                proc_macro2::TokenTree::Ident(ident) => {
                    Ok(ident.to_string())
                },
                _ => {
                    let message = "Expected ident for key of field";
                    spanned_error(symbols[0].span(), message)
                }
            }?;
            match &symbols[1] {
                proc_macro2::TokenTree::Punct(punct) => {
                    if punct.to_string() != "=" {
                        let message = format!("Expected an '=' operator, got '{punct}'");
                        spanned_error(symbols[1].span(), &message)?
                    }
                },
                _ => {
                    let message = "Expected identifier for key of field";
                    spanned_error(symbols[1].span(), message)?
                }
            };
            let value = match &symbols[2] {
                proc_macro2::TokenTree::Literal(literal) => {
                    Ok(literal.to_string().replace("\"", ""))
                },
                _ => {
                    let message = "Expected literal for value of field";
                    spanned_error(symbols[0].span(), message)
                }
            }?;
            let span = attribute.span();
    
            Ok( Self { attr, key, value, span } )
        }
        else {
            let message = format!("Could not parse metadata for attribute: {:#?}", &attribute.meta);
            spanned_error(attribute.meta.span(), &message)
        }
    }
}

/// Whatever Builder ends up needing, this will hold it in a convenient simplified struct
#[derive(Debug)]
pub struct BuilderStruct {
    name: syn::Ident,
    fields: Vec<syn::Field>,
    field_attr_each: HashMap<syn::Ident, KvFieldAttribute>,
}

impl BuilderStruct {
    pub fn new(context: &syn::DeriveInput) -> syn::Result<Self> {
        let name: syn::Ident = context.ident.clone();
        let fields: Vec<syn::Field> = match &context.data {
            syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                    syn::Result::Ok(named.into_iter().cloned().collect())
                }
                _ => spanned_error(
                    context.ident.clone().span(),
                    "'Builder' must be derived on a struct with named fields.",
                ),
            },
            _ => spanned_error(
                context.ident.clone().span(),
                "'Builder' must be derived on a struct with named fields.",
            ),
        }?;
        let mut field_attr_each: HashMap<syn::Ident, KvFieldAttribute> = HashMap::new();
        type KvFieldResult = Result<KvFieldAttribute, syn::Error>;
        for f in &fields {
            let (attrs, errs): (Vec<KvFieldResult>, Vec<KvFieldResult>)  = f
                .attrs
                .iter()
                .filter_map(|a| {
                    let attr = filter_field_attribute(a, "builder");
                    attr
                })
                .map(KvFieldAttribute::try_from)
                .partition(|r: &KvFieldResult| r.is_ok());

            let attrs: Vec<_> = attrs.into_iter().flatten().collect();
            let errs: Vec<_> = errs.into_iter().flat_map(|r| r.err()).collect();
            if let Some(err) = errs.first() {
                let syn_error = err.clone();
                syn::Result::Err(syn_error)?
            }
            

            let attr_each: Vec<_> = attrs
                .iter()
                .cloned()
                .filter(|a| a.key == "each")
                .collect();
            let attr_not_each: Vec<_> = attrs
                .iter()
                .cloned()
                .filter(|a| a.key != "each")
                .collect();
            if let Some(not_each_attr) = attr_not_each.first() {
                spanned_error(
                    not_each_attr.span, 
           "expected `builder(each = \"...\")`"
                )?;
            }
            


            match attr_each.len() {
                0 => {},
                1 => {
                    let field_attr = attr_each.first().clone().unwrap().clone();
                    field_attr_each.insert(f.ident.clone().unwrap(), field_attr.clone());     
                },
                _ => {
                    let message = "Expected at mosty one 'each' attribute on field.";
                    spanned_error(f.span(), message)?
                }
            }
        }

        let builder = BuilderStruct { name, fields, field_attr_each };
        syn::Result::Ok(builder)
    }

    pub fn create_field_builder_each_func(&self, ident: &syn::Ident, kv_field_attribute: &KvFieldAttribute) -> Option<proc_macro2::TokenStream> {
        if kv_field_attribute.key != "each" {
            return None;
        }

        let field_each_name = syn::Ident::new(&kv_field_attribute.value, proc_macro2::Span::call_site());
        let field = self.fields
            .iter()
            .filter(|f| f.ident.clone().unwrap().to_string() == ident.to_string())
            .next()
            .expect(&format!("Unknown identifier {} for attribute 'each'.", ident.to_string()));
        let field_type = field.ty.clone();
        let field_name = field.ident.clone().expect("Expected a named field");
        let inner = if let Some(inner) = get_vec_inner_type(&field_type) {
            inner 
        } else {
            return None;
        };

        let field_builder_each_func = quote::quote! { 
            fn #field_each_name(&mut self, #field_each_name: #inner) -> &mut Self {
                self.#field_name
                    .get_or_insert_with(::std::vec::Vec::new)
                    .push(#field_each_name);
                self
            }
        };

        Some(field_builder_each_func.into_token_stream())
    }

    pub fn create_field_builder_func(&self, field: &syn::Field) -> proc_macro2::TokenStream {
        let struct_name = self.name.clone();
        let field_name = field
            .ident
            .clone()
            .expect("Named fields only past this point");
        let field_type = field.ty.clone();

        let struct_name_string = struct_name.to_string();
        let field_name_string = field_name.to_string();
        let doc_lines = vec![
            format!(" Sets the `{field_name_string}` field of the `{struct_name_string}` builder."),
            String::new(),
            String::from(" Args:"),
            format!(
                " * `{field_name_string}` - Sets the value of the `{field_name_string}` field."
            ),
        ];
        let doc_attrs = doc_lines
            .iter()
            .map(|line| quote::quote! { #[doc = #line] });

        let field_builder_func = if let Some(inner) = get_option_inner_type(&field_type) {
            quote::quote! {
                #(#doc_attrs)*
                fn #field_name(&mut self, #field_name: #inner) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        } else {
            quote::quote! {
                #(#doc_attrs)*
                fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        };
        field_builder_func.into_token_stream()
    }

    pub fn create_build_func(&self) -> proc_macro2::TokenStream {
        // Collect basic info about the struct
        let struct_name = self.name.clone();
        let struct_name_string = struct_name.to_string();

        // Create docstring
        let doc_lines = vec![
            format!(" Builds a {struct_name_string} using the values provided to the builder."),
            String::new(),
            String::from(" Errors:"),
            format!(" * Returns an error if all fields have not been explicitly set."),
        ];
        let doc_attrs = doc_lines
            .iter()
            .map(|line| quote::quote! { #[doc = #line] });

        // Create lines of the form
        //
        //  let foo = self.foo.ok_or_else(|| { "foo has not been set".into() })?;
        //
        // Which will be used in our builder func
        let extract_field_values: Vec<proc_macro2::TokenStream> = self
            .fields
            .iter()
            .map(|f| {
                let field_name = f.ident.clone().expect("Expected named fields.");
                if get_option_inner_type(&f.ty).is_some() {
                    quote::quote! {
                        let #field_name = self.#field_name.clone().or(::std::option::Option::None);
                    }
                }
                else if get_vec_inner_type(&f.ty).is_some() {
                    quote::quote! {
                        let #field_name = self.#field_name.get_or_insert_with(::std::vec::Vec::new).clone();
                    }
                }
                else {
                    let errmsg = format!("{field_name} has not been set.");
                    quote::quote! {
                        let #field_name = self.#field_name.clone().ok_or_else(|| -> ::std::boxed::Box<dyn std::error::Error> {
                            #errmsg.into()
                        })?;
                    }
                }
            })
            .collect();
        // Create lines
        //
        //   foo,
        //   bar,
        //
        // to insert into the struct constructor
        //
        let struct_fields: Vec<proc_macro2::TokenStream> = self
            .fields
            .iter()
            .map(|f| {
                let field_name = f.ident.clone().expect("Expected named fields.");
                quote::quote! { #field_name, }
            })
            .collect();
        let build_func: proc_macro2::TokenStream = quote::quote! {
            #(#doc_attrs)*
            pub fn build(&mut self) -> ::std::result::Result<#struct_name, ::std::boxed::Box<dyn ::std::error::Error>> {
                #(#extract_field_values)*
                ::std::result::Result::Ok(#struct_name {
                    #(#struct_fields)*
                })
            }
        };
        build_func.into_token_stream()
    }
}

impl quote::ToTokens for BuilderStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = self.name.clone();
        let builder_name = syn::Ident::new(
            &format!("{}Builder", self.name),
            proc_macro2::Span::call_site(),
        );
        let each_func_names: Vec<String> = self
            .field_attr_each
            .values()
            .map(|kv| kv.value.clone())
            .collect();
        let field_builder_funcs: Vec<proc_macro2::TokenStream> = self
            .fields
            .iter()
            .filter(|f| {
                let field_name = f.ident.clone().unwrap().to_string();
                !each_func_names.contains(&field_name)
            })
            .map(|f| self.create_field_builder_func(f))
            .collect();
        let field_builder_each_funcs: Vec<proc_macro2::TokenStream> = self
            .field_attr_each
            .iter()
            .flat_map(|(ident,kv_attr_field)| {
                self.create_field_builder_each_func(ident, kv_attr_field) 
            })
            .collect();
        let builder_docstring = format!(" Builder for {struct_name}.");
        let builder_impl_docstring = format!(" Creates a {builder_name} struct for the object.");
        let build_func = self.create_build_func();
        let generated_tokens: proc_macro2::TokenStream = quote::quote! {
            #[doc = #builder_docstring]
            pub struct #builder_name {
                executable: ::std::option::Option<::std::string::String>,
                args: ::std::option::Option<std::vec::Vec<::std::string::String>>,
                env: ::std::option::Option<::std::vec::Vec<std::string::String>>,
                current_dir: ::std::option::Option<std::string::String>,
            }

            impl #struct_name {
                #[doc = #builder_impl_docstring]
                pub fn builder() -> #builder_name {
                    #builder_name {
                        executable: None,
                        args: None,
                        env: None,
                        current_dir: None,
                    }
                }
            }

            impl #builder_name {
                #(#field_builder_funcs)*

                #(#field_builder_each_funcs)*

                #build_func
            }
        };
        tokens.extend(generated_tokens);
    }
}

pub fn derive(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // Derive the input from the user struct
    // Fail early if user gives bad input
    let context: syn::DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    // Collect relevant information to implement builder for struct
    let builder: BuilderStruct = match BuilderStruct::new(&context) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    // Generate the block of code to implement struct
    builder.into_token_stream()
}
