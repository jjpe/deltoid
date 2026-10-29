//!

use proc_macro2::{
    Delimiter as Delimiter2,
    TokenTree as TokenTree2
};
use crate::{DeriveError, DeriveResult};
use syn::*;


/// A `field` in the input struct or input enum variant
/// is marked with #[delta(ignore_field)].
///
/// NOTE: a `#[delta(..)]` attribute whose argument is anything other
///       than exactly `ignore_field` (e.g. a typo like `ignore`) is an
///       error, not a silently-ignored no-op: a field the user meant
///       to exclude must never be delta-encoded by accident.
pub(crate) fn ignore_field(field: &Field) -> DeriveResult<bool> {
    let mut ignore = false;
    for attr in field.attrs.iter() {
        let attr_segments: Vec<String> = attr.path.segments.iter()
            .map(|path_segment| format!("{}", path_segment.ident))
            .collect();
        let is_delta_attr = attr_segments == &["delta"];
        if !is_delta_attr {
            continue;
        }
        const DELIM: Delimiter2 = Delimiter2::Parenthesis;
        let arg_tokens_iter = attr.tokens.clone().into_iter().next();
        let found: String = match arg_tokens_iter {
            Some(TokenTree2::Group(g)) if g.delimiter() == DELIM =>
                g.stream().clone().into_iter()
                    .map(|tt| format!("{}", tt))
                    .collect::<Vec<String>>()
                    .join(" "),
            Some(tt) => format!("{}", tt),
            None => String::new(),
        };
        if found == "ignore_field" {
            ignore = true;
        } else {
            return Err(DeriveError::UnknownDeltaAttribute { found });
        }
    }
    Ok(ignore)
}
