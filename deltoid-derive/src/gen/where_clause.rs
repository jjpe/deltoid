//!

use crate::DeriveResult;
use crate::gen::InputType;
use proc_macro2::{Span as Span2};
use syn::*;
use syn::punctuated::*;
use syn::token::Comma;


#[inline(always)]
pub(crate) fn empty() -> WhereClause {
    WhereClause {
        where_token: Token![where](Span2::call_site()),
        predicates: Punctuated::new(),
    }
}

/// Adapt the input `WhereClause` in `self` for use in
/// the definition of the generated Delta type.
pub(crate) fn for_delta_type_definition(
    base: &WhereClause,
    input: &InputType,
) -> DeriveResult<WhereClause> {
    let mut clause: WhereClause = base.clone();
    for type_param_decl in input.type_param_decls().iter() {
        // NOTE: <type_param_decl>:  Clone + PartialEq + ...
        clause.predicates.push(WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: type_param_decl_to_type(type_param_decl)?,
            colon_token: Token![:](Span2::call_site()),
            bounds: vec![ // Add type param bounds
                trait_bound![Clone],
                trait_bound![PartialEq],
                trait_bound![std::fmt::Debug],
                trait_bound![deltoid::Core],
            ].into_iter().collect(),
        }));
    }
    Ok(clause)
}

pub(crate) fn for_deltoid_trait_impl(
    base: &WhereClause,
    input: &InputType,
    generic_params: &Punctuated<GenericParam, Comma>,
) -> DeriveResult<WhereClause> {
    let mut clause: WhereClause = base.clone();
    for type_param_decl in generic_params.iter() {
        // NOTE: <type_param_decl>:  Clone + PartialEq + ...
        clause.predicates.push(WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: type_param_decl_to_type(type_param_decl)?,
            colon_token: Token![:](Span2::call_site()),
            bounds: vec![ // Add type param bounds
                trait_bound![Clone],
                trait_bound![PartialEq],
                trait_bound![std::fmt::Debug],
                trait_bound![deltoid::Core],
                trait_bound![deltoid::FromDelta],
                trait_bound![deltoid::IntoDelta],
                trait_bound![serde::Serialize],
                trait_bound![serde::Deserialize<'de>],
            ].into_iter().collect(),
        }));
    }
    Ok(clause)
}

fn type_param_decl_to_type(
    type_param_decl: &GenericParam
) -> DeriveResult<Type> {
    let type_param = match type_param_decl {
        GenericParam::Lifetime(_) => unimplemented!("GenericParam::Lifetime(_)"),
        GenericParam::Const(_)    => unimplemented!("GenericParam::Const(_)"),
        GenericParam::Type(type_param) => type_param,
    };
    Ok(Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: vec![PathSegment {
                ident: type_param.ident.clone(),
                arguments: PathArguments::None,
            }].into_iter().collect()
        },
    }))
}
