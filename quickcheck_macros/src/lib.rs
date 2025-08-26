use std::mem;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    parse_quote,
    spanned::Spanned,
};

#[proc_macro_attribute]
pub fn quickcheck(_args: TokenStream, input: TokenStream) -> TokenStream {
    let output = match syn::Item::parse.parse(input.clone()) {
        Ok(syn::Item::Fn(mut item_fn)) => {
            let mut inputs = syn::punctuated::Punctuated::new();
            let mut errors = Vec::new();

            item_fn.sig.inputs.iter().for_each(|input| match *input {
                syn::FnArg::Typed(syn::PatType { ref ty, .. }) => {
                    inputs.push(parse_quote!(_: #ty));
                }
                _ => errors.push(syn::parse::Error::new(
                    input.span(),
                    "unsupported kind of function argument",
                )),
            });

            if errors.is_empty() {
                let attrs = mem::take(&mut item_fn.attrs);
                let name = &item_fn.sig.ident;
                if let Some(variadic) = &item_fn.sig.variadic {
                    // variadics are just for `extern fn`
                    errors.push(syn::parse::Error::new(
                        variadic.span(),
                        "unsupported variadic",
                    ));
                }
                let fn_type = syn::TypeBareFn {
                    lifetimes: None,
                    unsafety: item_fn.sig.unsafety,
                    abi: item_fn.sig.abi.clone(),
                    fn_token: <syn::Token![fn]>::default(),
                    paren_token: syn::token::Paren::default(),
                    inputs,
                    variadic: None,
                    output: item_fn.sig.output.clone(),
                };

                quote! {
                    #[test]
                    #(#attrs)*
                    fn #name() {
                        #item_fn
                       ::quickcheck::quickcheck(#name as #fn_type)
                    }
                }
            } else {
                errors
                    .iter()
                    .map(syn::parse::Error::to_compile_error)
                    .collect()
            }
        }
        Ok(syn::Item::Static(mut item_static)) => {
            let attrs = mem::take(&mut item_static.attrs);
            let name = &item_static.ident;

            quote! {
                #[test]
                #(#attrs)*
                fn #name() {
                    #item_static
                    ::quickcheck::quickcheck(#name)
                }
            }
        }
        _ => {
            let span = proc_macro2::TokenStream::from(input).span();
            let msg =
                "#[quickcheck] is only supported on statics and functions";

            syn::parse::Error::new(span, msg).to_compile_error()
        }
    };

    output.into()
}

use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// This is the function that will be executed by the compiler when it sees `#[derive(Arbitrary)]`.
#[proc_macro_derive(Arbitrary)]
pub fn arbitrary_derive(input: TokenStream) -> TokenStream {
    // 1. Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct (e.g., `ReverseArgs`).
    let name = &input.ident;

    // Handle generics on the struct (e.g., `MyStruct<T>`).
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 2. Generate the body of the `arbitrary` method.
    let arbitrary_body = gen_arbitrary_body(&input.data);
    
    // 3. Generate the body of the `shrink` method.
    let shrink_body = gen_shrink_body(&input.data);

    // 4. Build the final `impl` block.
    // We use `::quickcheck::Arbitrary` to make the path unambiguous.
    let expanded = quote! {
        // Add `#[allow(unused_qualifications)]` to handle paths gracefully.
        #[allow(unused_qualifications)]
        impl #impl_generics ::quickcheck::Arbitrary for #name #ty_generics #where_clause {
            fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
                #arbitrary_body
            }

            fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                #shrink_body
            }
        }
    };

    // 5. Return the generated code as a TokenStream.
    TokenStream::from(expanded)
}

/// Helper function to generate the `arbitrary` method's implementation.
fn gen_arbitrary_body(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // For `struct Foo { a: u32, b: bool }`
                    // generates: `a: u32::arbitrary(g), b: bool::arbitrary(g)`
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote! {
                            #name: <#ty as ::quickcheck::Arbitrary>::arbitrary(g)
                        }
                    });
                    quote! {
                        Self {
                            #(#recurse),*
                        }
                    }
                }
                _ => unimplemented!("Derive Arbitrary only supports structs with named fields."),
            }
        }
        _ => unimplemented!("Derive Arbitrary can only be used on structs."),
    }
}

/// Helper function to generate the `shrink` method's implementation.
fn gen_shrink_body(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let field_shrinks = fields.named.iter().map(|f| {
                        let field_name = f.ident.as_ref().unwrap();
                        
                        // For each field, create a clone of all *other* fields.
                        let other_fields_clone = fields.named.iter().filter_map(|f2| {
                            let f2_name = f2.ident.as_ref().unwrap();
                            if f2_name == field_name {
                                None
                            } else {
                                Some(quote! { let #f2_name = self.#f2_name.clone(); })
                            }
                        });

                        // Create the struct instance with the new shrunk field value.
                        let struct_construction = fields.named.iter().map(|f2| {
                            let f2_name = f2.ident.as_ref().unwrap();
                            if f2_name == field_name {
                                quote! { #f2_name: new_value }
                            } else {
                                quote! { #f2_name: #f2_name.clone() }
                            }
                        });

                        // The final map expression for this one field.
                        quote! {
                            self.#field_name.shrink().map({
                                #(#other_fields_clone)*
                                move |new_value| {
                                    Self {
                                        #(#struct_construction),*
                                    }
                                }
                            })
                        }
                    });

                    // Chain all the individual field shrinkers together.
                    quote! {
                        Box::new(
                            // Start with an empty iterator and chain onto it.
                            ::std::iter::empty()
                                #( .chain(#field_shrinks) )*
                        )
                    }
                }
                _ => unimplemented!("Derive Arbitrary only supports structs with named fields."),
            }
        }
        _ => unimplemented!("Derive Arbitrary can only be used on structs."),
    }
}