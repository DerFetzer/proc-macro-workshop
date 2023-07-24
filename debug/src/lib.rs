use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Expr, ExprLit, Fields, Lit, LitStr,
    Meta, MetaNameValue,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    eprintln!("{input:#?}");

    let ident = input.ident;

    let struct_fields = if let Data::Struct(ds) = input.data {
        if let Fields::Named(fields) = ds.fields {
            fields.named
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    };

    let field_setters = struct_fields.iter().enumerate().map(|(i, f)| {
        let name = &f.ident;
        let name_str = LitStr::new(
            &name.as_ref().map(|n| n.to_string()).unwrap_or_default(),
            Span::call_site(),
        );
        let debug_attr = f
            .attrs
            .iter()
            .find(|a| {
                if let Meta::NameValue(nv) = &a.meta {
                    nv.path.is_ident("debug")
                } else {
                    false
                }
            })
            .map(|a| match &a.meta {
                Meta::NameValue(nv) => nv,
                _ => unreachable!(),
            });
        let format_string = match debug_attr {
            Some(named_value) => {
                if let MetaNameValue {
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }),
                    ..
                } = named_value
                {
                    format!("{}: {}", name_str.value(), lit_str.value())
                } else {
                    unimplemented!()
                }
            }
            None => format!("{}: {{:?}}", name_str.value()),
        };
        let mut stream = quote_spanned! {f.span() =>
            fmt.write_fmt(::std::format_args!(#format_string, self.#name))?;
        };
        if i != struct_fields.len() - 1 {
            stream.extend(quote! {
                fmt.write_str(", ")?;
            })
        }
        stream
    });

    let struct_start = format!("{} {{ ", ident);
    let struct_end = " }";

    let output = quote! {
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.write_str(#struct_start)?;
                #(#field_setters)*
                fmt.write_str(#struct_end)
            }
        }
    };
    eprintln!("{output}");
    output.into()
}
