use defmt_parser::ParserMode;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;

use crate::{construct, function_like::log};

pub(crate) fn expand(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as log::Args);

    let format_string = args.format_string.value();
    let fragments = match defmt_parser::parse(&format_string, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => abort!(args.format_string, "{}", e),
    };

    let formatting_exprs = args
        .formatting_args
        .map(|punctuated| punctuated.into_iter().collect())
        .unwrap_or_else(Vec::new);

    let log::Codegen { patterns, exprs } = log::Codegen::new(
        &fragments,
        formatting_exprs.len(),
        args.format_string.span(),
    );

    let header = construct::interned_string(&format_string, "println", false);
    quote!({
        match (#(&(#formatting_exprs)),*) {
            (#(#patterns),*) => {
                defmt::export::acquire();
                defmt::export::header(&#header);
                #(#exprs;)*
                defmt::export::release()
            }
        }
    })
    .into()
}
