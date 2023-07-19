use std::ops::Range;

use proc_macro2::{Delimiter, Group, Literal, TokenStream, TokenTree};
use syn::{braced, parse::Parse, parse_macro_input, Ident, LitInt, Token};

#[derive(Debug)]
struct Seq {
    counter_ident: Ident,
    lit_from: LitInt,
    lit_to: LitInt,
    inclusive: bool,
    content: proc_macro2::TokenStream,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let counter_ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let lit_from = input.parse()?;
        let inclusive = if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            true
        } else {
            input.parse::<Token![..]>()?;
            false
        };
        let lit_to = input.parse()?;

        let content;
        braced!(content in input);
        let content = content.parse()?;

        Ok(Self {
            counter_ident,
            lit_from,
            lit_to,
            inclusive,
            content,
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Seq);
    eprintln!("{input:#?}");

    let mut output = TokenStream::new();

    let from = input.lit_from.base10_parse::<usize>().unwrap();
    let to = input.lit_to.base10_parse::<usize>().unwrap();

    let range = if input.inclusive {
        from..to + 1
    } else {
        from..to
    };

    // eprintln!("{from} {to}");

    if has_repetition_delimiter(input.content.clone()) {
        output.extend(repeat_sections(&input.counter_ident, range, input.content))
    } else {
        for i in range {
            let current_stream =
                replace_ident(&input.counter_ident.clone(), i, input.content.clone());
            // eprintln!("{i} {current_stream:#?}");
            output.extend(current_stream);
        }
    }

    // eprintln!("Output: {output:#?}");
    eprintln!("Output: {}", output);
    output.into()
}

fn has_repetition_delimiter(stream: TokenStream) -> bool {
    let tokens = Vec::from_iter(stream.into_iter());

    for i in 0..tokens.len() {
        match &tokens[i..] {
            [TokenTree::Punct(pound), TokenTree::Group(group), TokenTree::Punct(asterisk)]
                if pound.as_char() == '#'
                    && asterisk.as_char() == '*'
                    && group.delimiter() == Delimiter::Parenthesis =>
            {
                return true;
            }
            [TokenTree::Group(group), ..] => {
                if has_repetition_delimiter(group.stream()) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn repeat_sections(ident: &Ident, range: Range<usize>, content: TokenStream) -> TokenStream {
    let content_tokens: Vec<_> = Vec::from_iter(content);
    let mut replaced_tokens = Vec::with_capacity(content_tokens.len());

    let mut i = 0;
    while i < content_tokens.len() {
        let replaced_token = match &content_tokens[i..] {
            [TokenTree::Punct(pound), TokenTree::Group(group), TokenTree::Punct(asterisk)]
                if pound.as_char() == '#'
                    && asterisk.as_char() == '*'
                    && group.delimiter() == Delimiter::Parenthesis =>
            {
                i += 2;
                TokenTree::Group(Group::new(
                    Delimiter::None,
                    TokenStream::from_iter(
                        range
                            .clone()
                            .map(|i| replace_ident(ident, i, group.stream())),
                    ),
                ))
            }
            [TokenTree::Group(group), ..] => TokenTree::Group(Group::new(
                group.delimiter(),
                repeat_sections(ident, range.clone(), group.stream()),
            )),
            [tt, ..] => tt.clone(),
            _ => unreachable!(),
        };
        replaced_tokens.push(replaced_token);
        i += 1;
    }

    TokenStream::from_iter(replaced_tokens)
}

fn replace_ident(ident: &Ident, value: usize, content: TokenStream) -> TokenStream {
    let content_tokens: Vec<_> = Vec::from_iter(content);
    let mut replaced_tokens = Vec::with_capacity(content_tokens.len());

    let mut i = 0;
    while i < content_tokens.len() {
        let replaced_token = match &content_tokens[i..] {
            [TokenTree::Ident(first), TokenTree::Punct(delim), TokenTree::Ident(current_ident), ..]
                if delim.as_char() == '~' && ident == current_ident =>
            {
                i += 2;
                TokenTree::Ident(Ident::new(&format!("{}{}", first, value), first.span()))
            }
            [TokenTree::Ident(current_ident), ..] if ident == current_ident => {
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
