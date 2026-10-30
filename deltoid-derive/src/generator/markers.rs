//!

use proc_macro2::{
    Delimiter as Delimiter2,
    Group as Group2
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
        let attr_segments: Vec<String> = attr.path().segments.iter()
            .map(|path_segment| format!("{}", path_segment.ident))
            .collect();
        let is_delta_attr = attr_segments == &["delta"];
        if !is_delta_attr {
            continue;
        }
        const DELIM: Delimiter2 = Delimiter2::Parenthesis;
        // NOTE: syn >= 2 stores the parsed attribute in `attr.meta`; recover
        //       the same `found` strings the syn 1 `attr.tokens` inspection
        //       produced: the space-joined inner tokens of a paren-delimited
        //       list, the display form of a non-paren group, "" for a bare
        //       path attribute and "=" for a name-value attribute.
        let found: String = match &attr.meta {
            Meta::List(list) => {
                let delim: Delimiter2 = match &list.delimiter {
                    MacroDelimiter::Paren(_)   => Delimiter2::Parenthesis,
                    MacroDelimiter::Brace(_)   => Delimiter2::Brace,
                    MacroDelimiter::Bracket(_) => Delimiter2::Bracket,
                };
                if delim == DELIM {
                    list.tokens.clone().into_iter()
                        .map(|tt| format!("{}", tt))
                        .collect::<Vec<String>>()
                        .join(" ")
                } else {
                    format!("{}", Group2::new(delim, list.tokens.clone()))
                }
            },
            Meta::Path(_) => String::new(),
            Meta::NameValue(_) => String::from("="),
        };
        if found == "ignore_field" {
            ignore = true;
        } else {
            return Err(DeriveError::UnknownDeltaAttribute { found });
        }
    }
    Ok(ignore)
}
