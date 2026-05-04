use quote::ToTokens;

/// Whatever Builder ends up needing, this will hold it in a convenient simplified struct
pub struct BuilderStruct {
    name: String,
}

impl BuilderStruct {
    pub fn new(context: syn::DeriveInput) -> syn::Result<Self> {
        let name = context.ident.to_string();
        let builder = BuilderStruct { name };
        syn::Result::Ok(builder)
    }
}

impl quote::ToTokens for BuilderStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = syn::Ident::new(&self.name, proc_macro2::Span::call_site());
        let builder_name = syn::Ident::new(
            &format!("{}Builder", self.name),
            proc_macro2::Span::call_site(),
        );
        let mut generated_tokens: proc_macro2::TokenStream = quote::quote! {
            pub struct #builder_name {
                executable: Option<String>,
                args: Option<Vec<String>>,
                env: Option<Vec<String>>,
                current_dir: Option<String>,
            }

            /// Defines a common implementation for builder
            impl #struct_name {
                pub fn builder() -> #builder_name {
                    #builder_name {
                        executable: None,
                        args: None,
                        env: None,
                        current_dir: None,
                    }
                }
            }
        };
        tokens.to_tokens(&mut generated_tokens)
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
    let builder: BuilderStruct = match BuilderStruct::new(context) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    // Generate the block of code to implement struct
    // let output: proc_macro2::TokenStream = builder.into_token_stream();
    // output;
    quote::quote! { #builder }
}
