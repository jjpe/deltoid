//! This module allows creating trait bounds easily.

use proc_macro2::{Span as Span2};
use syn::*;
use syn::punctuated::*;
use syn::token::Comma;
use quote::{format_ident};

macro_rules! trait_bound {
    ($($path_segment:ident)::+ <$lifetime:lifetime>) => {
        $crate::gen::trait_bounds::lifetimed_trait_bound(
            &[ $( stringify!($path_segment) ),* ],
            stringify!($lifetime)
        )
    };
    ($($path_segment:ident)::+) => {
        $crate::gen::trait_bounds::trait_bound(&[
            $( stringify!($path_segment) ),*
        ])
    };
}


/// Create a trait bound based on the provided `path_segments` and `lifetime`
/// e.g. `trait_bound(&["foo", "Bar"], "lifetime")` will create a trait
/// bound `foo::Bar<'lifetime>`.
pub(crate) fn lifetimed_trait_bound(
    path_segments: &[&str],
    lifetime: &str
) -> TypeParamBound {
    fn generic_lifetime_arg(lifetime: &str) -> PathArguments {
        PathArguments::AngleBracketed(
            AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token![<](Span2::call_site()),
                args: {
                    let mut p: Punctuated<GenericArgument, Comma> =
                        Punctuated::new();
                    p.push(GenericArgument::Lifetime(Lifetime {
                        apostrophe: Span2::call_site(),
                        ident: format_ident!("{}", lifetime),
                    }));
                    p
                },
                gt_token: Token![>](Span2::call_site()),
            }
        )
    }
    TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: Some(BoundLifetimes {
            for_token: Token![for](Span2::call_site()),
            lt_token: Token![<](Span2::call_site()),
            lifetimes: {
                let mut punctuated: Punctuated<LifetimeDef, Comma> =
                    Punctuated::new();
                punctuated.push(LifetimeDef::new(Lifetime {
                    apostrophe: Span2::call_site(),
                    ident: format_ident!("{}", lifetime),
                }));
                punctuated
            },
            gt_token: Token![>](Span2::call_site()),
        }),
        path: Path {
            leading_colon: None,
            segments: {
                let segment_count = path_segments.len();
                path_segments.iter()
                    .enumerate()
                    .map(|(idx, segment)| PathSegment {
                        ident: Ident::new(segment, Span2::call_site()),
                        arguments: if idx < segment_count - 1 {
                            PathArguments::None
                        } else { // Add the lifetime arg to the last segment
                            generic_lifetime_arg(lifetime)
                        },
                    })
                    .collect()
            },
        },
    })
}

/// Create a trait bound based on the provided `path_segments` e.g.
/// `trait_bound(&["foo", "Bar"])` will create a trait bound `foo::Bar`.
pub(crate) fn trait_bound(path_segments: &[&str]) -> TypeParamBound {
    TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path {
            leading_colon: None,
            segments: path_segments.iter()
                .map(|segment| PathSegment {
                    arguments: PathArguments::None,
                    ident: Ident::new(segment, Span2::call_site()),
                })
                .collect(),
        },
    })
}
