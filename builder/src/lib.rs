use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Expr, Fields,
    GenericArgument, Ident, Lit, PathArguments, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    // eprintln!("Input: {:#?}", input);
    let input = parse_macro_input!(input as DeriveInput);
    // eprintln!("Parsed input: {:#?}", input);

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
        let ty = get_generic_type(&f.ty, "Option").unwrap_or(&f.ty);
        quote_spanned! {f.span() =>
            #name: ::std::option::Option<#ty>
        }
    });
    let builder_struct_init_fields = struct_fields.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {f.span() =>
            #name: ::std::option::Option::None
        }
    });
    let builder_struct_impl = struct_fields.iter().map(|f| {
        let name = &f.ident;
        let ty = get_generic_type(&f.ty, "Option").unwrap_or(&f.ty);
        let setter = quote_spanned! {f.span() =>
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = ::std::option::Option::Some(#name);
                self
            }
        };
        match get_each_builder_attribute(&f.attrs) {
            Ok(Some(each)) if each != name.as_ref().map(|n| n.to_string()).unwrap_or_default() => {
                let each_ident = Ident::new(&each, Span::call_site());
                let each_ty = get_generic_type(&f.ty, "Vec");
                quote_spanned! {f.span() =>
                    #setter

                    fn #each_ident(&mut self, #each_ident: #each_ty) -> &mut Self {
                        self.#name.get_or_insert(::std::vec::Vec::new()).push(#each_ident);
                        self
                    }
                }
            }
            Err(e) => e.into_compile_error(),
            _ => setter,
        }
    });
    let builder_struct_build_set_fields = struct_fields.iter().map(|f| {
        let name = &f.ident;
        let ty = get_generic_type(&f.ty, "Option");
        if ty.is_some() {
            quote_spanned! {f.span() =>
                #name: self.#name.take(),
            }
        }
        else if let Ok(Some(_)) = get_each_builder_attribute(&f.attrs) {
            quote_spanned! {f.span() =>
                #name: self.#name.take().unwrap_or_default(),
            }
        }
        else {
            let error_msg = format!("{} is not set!", name.as_ref().map(|n| n.to_string()).unwrap());
            quote_spanned! {f.span() =>
                #name: self.#name.take().ok_or(::std::boxed::Box::<dyn ::std::error::Error>::from(#error_msg))?,
            }
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

fn get_generic_type<'a>(ty: &'a Type, ty_ident: &'_ str) -> Option<&'a Type> {
    if let Type::Path(path_ty) = ty {
        match path_ty.path.segments.last()? {
            segment if segment.ident == ty_ident => {
                if let PathArguments::AngleBracketed(arg) = &segment.arguments {
                    if let GenericArgument::Type(ty) = arg.args.first()? {
                        Some(ty)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

fn get_each_builder_attribute(attrs: &[Attribute]) -> Result<Option<String>, syn::Error> {
    let builder_attr = attrs.iter().find(|a| a.path().is_ident("builder"));
    let builder_attr = if let Some(builder_attr) = builder_attr {
        builder_attr
    } else {
        return Ok(None);
    };
    let args: Expr = builder_attr
        .parse_args()
        .expect("Invalid syntax for builder attribute!");
    if let Expr::Assign(assign) = args {
        if let (Expr::Path(each), Expr::Lit(expr_lit)) = (*assign.left, *assign.right) {
            if !each.path.is_ident("each") {
                return Err(syn::Error::new_spanned(
                    &builder_attr.meta,
                    "expected `builder(each = \"...\")`",
                ));
            }
            if let Lit::Str(lit_str) = expr_lit.lit {
                Ok(Some(lit_str.value()))
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
}
