//!

use proc_macro2::{
    Delimiter as Delimiter2,
    TokenTree as TokenTree2
};
use syn::*;


/// A `field` in the input struct or input enum variant
/// is marked with #[delta(ignore_field)].
pub(crate) fn ignore_field(field: &Field) -> bool {
    let mut ignore = false;
    for attr in field.attrs.iter() {
        let attr_segments: Vec<String> = attr.path.segments.iter()
            .map(|path_segment| format!("{}", path_segment.ident))
            .collect();
        let is_delta_attr = attr_segments == &["delta"];
        let arg_tokens_iter = attr.tokens.clone().into_iter().next();
        const DELIM: Delimiter2 = Delimiter2::Parenthesis;
        let arg_is_ignore_field = match arg_tokens_iter {
            Some(TokenTree2::Group(g)) if g.delimiter() == DELIM => {
                let tokens: Vec<String> = g.stream().clone().into_iter()
                    .map(|tt| format!("{}", tt))
                    .collect();
                tokens == &["ignore_field"]
            },
            _ => false,
        };
        ignore = ignore || is_delta_attr && arg_is_ignore_field
    }
    ignore
}
