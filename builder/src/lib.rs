use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // eprintln!("Input: {:#?}", input);
    let input = parse_macro_input!(input as DeriveInput);
    eprintln!("Parsed input: {:#?}", input);

    let struct_name = input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());

    let struct_fields = if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(fields) = data_struct.fields {
            fields.named
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    };

    let builder_struct_fields = struct_fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote_spanned! {f.span() =>
            #name: Option<#ty>
        }
    });
    let builder_struct_init_fields = struct_fields.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {f.span() =>
            #name: None
        }
    });
    let builder_struct_impl = struct_fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote_spanned! {f.span() =>
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    let builder_struct_build_set_fields = struct_fields.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {f.span() =>
            #name: self.#name.take().ok_or(::std::boxed::Box::<dyn ::std::error::Error>::from("name is not set!".to_string()))?,
        }
    });

    let struct_impl = quote! {
        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_struct_init_fields),*
                }
            }
        }
    };

    let builder_struct = quote! {
        pub struct #builder_name {
            #(#builder_struct_fields),*
        }
    };

    let builder_impl = quote! {
        impl #builder_name {
            #(#builder_struct_impl)*

            pub fn build(&mut self) -> ::std::result::Result<#struct_name, ::std::boxed::Box<dyn ::std::error::Error>> {
                Ok(
                    #struct_name{
                        #(#builder_struct_build_set_fields)*
                    }
                )
            }
        }
    };
    quote! {
        #struct_impl
        #builder_struct
        #builder_impl
    }
    .into()
}
