use quote::ToTokens;

/// Creates a `syn::Result::Err` for a particular span with a display message
pub fn spanned_error<T>(span: proc_macro2::Span, message: &str) -> syn::Result<T> {
    syn::Result::Err(syn::Error::new(span, message))
}

#[allow(dead_code)]
fn is_option_type(syntype: &syn::Type) -> bool {
    match syntype {
        syn::Type::Path(syn::TypePath { path, .. }) => match path { 
            _ => todo!("Is a type path") 
        },
        _ => todo!("Not a type path")
    }
}

fn get_option_inner_type(syntype: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path}) = syntype
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

/// Whatever Builder ends up needing, this will hold it in a convenient simplified struct
pub struct BuilderStruct {
    name: syn::Ident,
    fields: Vec<syn::Field>,
}

impl BuilderStruct {
    pub fn new(context: &syn::DeriveInput) -> syn::Result<Self> {
        let name = context.ident.clone();
        let fields = match &context.data {
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
        let builder = BuilderStruct { name, fields };
        syn::Result::Ok(builder)
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
        }
        else 
        {
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
            pub fn build(&mut self) -> ::std::result::Result<#struct_name, Box<dyn ::std::error::Error>> {
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
        // let struct_name = syn::Ident::new(&self.name, proc_macro2::Span::call_site());
        let builder_name = syn::Ident::new(
            &format!("{}Builder", self.name),
            proc_macro2::Span::call_site(),
        );
        let field_builder_funcs: Vec<proc_macro2::TokenStream> = self
            .fields
            .iter()
            .map(|f| self.create_field_builder_func(f))
            .collect();
        let builder_docstring = format!(" Builder for {struct_name}.");
        let builder_impl_docstring = format!(" Creates a {builder_name} struct for the object.");
        let build_func = self.create_build_func();
        let generated_tokens: proc_macro2::TokenStream = quote::quote! {
            #[doc = #builder_docstring]
            pub struct #builder_name {
                executable: Option<String>,
                args: Option<Vec<String>>,
                env: Option<Vec<String>>,
                current_dir: Option<String>,
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
