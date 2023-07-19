use proc_macro::{Group, Literal, Span, TokenStream, TokenTree};
use syn::{braced, parse::Parse, parse_macro_input, token, Ident, LitInt, Token};

#[allow(unused)]
#[derive(Debug)]
struct Seq {
    counter_ident: Ident,
    in_token: Token![in],
    lit_from: LitInt,
    range_token: Token![..],
    lit_to: LitInt,
    brace_token: token::Brace,
    content: proc_macro2::TokenStream,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            counter_ident: input.parse()?,
            in_token: input.parse()?,
            lit_from: input.parse()?,
            range_token: input.parse()?,
            lit_to: input.parse()?,
            brace_token: braced!(content in input),
            content: content.parse()?,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Seq);
    eprintln!("{input:#?}");

    let mut output = TokenStream::new();

    let from = input.lit_from.base10_parse::<usize>().unwrap();
    let to = input.lit_to.base10_parse::<usize>().unwrap();

    eprintln!("{from} {to}");

    for i in from..to {
        let current_stream = replace_ident(
            &input.counter_ident.clone(),
            i,
            input.content.clone().into(),
        );
        eprintln!("{i} {current_stream:#?}");
        output.extend(current_stream);
    }

    eprintln!("Output: {output:#?}");
    output
}

fn replace_ident(ident: &Ident, value: usize, content: TokenStream) -> TokenStream {
    let content_tokens: Vec<_> = Vec::from_iter(content);
    let mut replaced_tokens = Vec::with_capacity(content_tokens.len());

    let mut i = 0;
    while i < content_tokens.len() {
        let replaced_token = match &content_tokens[i..] {
            [TokenTree::Ident(first), TokenTree::Punct(delim), TokenTree::Ident(current_ident), ..]
                if delim.as_char() == '~' && *ident == current_ident.to_string() =>
            {
                i += 2;
                TokenTree::Ident(proc_macro::Ident::new(
                    &format!("{}{}", first, value),
                    Span::call_site(),
                ))
            }
            [TokenTree::Ident(current_ident), ..] if *ident == current_ident.to_string() => {
                TokenTree::Literal(Literal::usize_unsuffixed(value))
            }
            [TokenTree::Group(group), ..] => TokenTree::Group(Group::new(
                group.delimiter(),
                replace_ident(ident, value, group.stream())
                    .into_iter()
                    .collect(),
            )),
            [tt, ..] => tt.clone(),
            _ => unreachable!(),
        };
        replaced_tokens.push(replaced_token);
        i += 1;
    }

    TokenStream::from_iter(replaced_tokens)
}
