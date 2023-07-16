use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, GenericArgument, Ident,
    PathArguments, Type,
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
        let ty = get_type_from_option(&f.ty).unwrap_or(&f.ty);
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
        let ty = get_type_from_option(&f.ty).unwrap_or(&f.ty);
        quote_spanned! {f.span() =>
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    let builder_struct_build_set_fields = struct_fields.iter().map(|f| {
        let name = &f.ident;
        let error_msg = format!("{} is not set!", name.clone().unwrap());
        let ty = get_type_from_option(&f.ty);
        if ty.is_some() {
            quote_spanned! {f.span() =>
                #name: self.#name.take(),
            }
        }
        else {
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

fn get_type_from_option(ty: &Type) -> Option<&Type> {
    if let Type::Path(path_ty) = ty {
        match path_ty.path.segments.last()? {
            segment if segment.ident == "Option" => {
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
