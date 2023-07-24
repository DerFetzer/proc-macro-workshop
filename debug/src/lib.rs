use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    eprintln!("{input:#?}");

    let ident = input.ident;
    let ident_str = LitStr::new(&ident.to_string(), Span::call_site());

    let struct_fields = if let Data::Struct(ds) = input.data {
        if let Fields::Named(fields) = ds.fields {
            fields.named
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    };

    let field_setters = struct_fields.iter().map(|f| {
        let name = &f.ident;
        let name_str = LitStr::new(&ident.to_string(), Span::call_site());
        quote_spanned! {f.span() =>
            .field(#name_str, &self.#name)
        }
    });

    let output = quote! {
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.debug_struct(#ident_str)
                    #(#field_setters)*
                    .finish()
            }
        }
    };
    eprintln!("{output}");
    output.into()
}
