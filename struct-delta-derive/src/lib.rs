extern crate proc_macro;

#[macro_use] mod error;

use crate::error::{DeriveResult};
use proc_macro::TokenStream;
use proc_macro2::{
    Literal as Literal2, Punct as Punct2, Spacing as Spacing2,
    Span as Span2,
    TokenTree as TokenTree2, TokenStream as TokenStream2
};
use quote::{format_ident, quote};
use std::iter::FromIterator;
use syn::{
    parse_macro_input, Token/* macro that expands to a type, not a type */,
    // Attribute,
    Data, DataStruct, DataEnum, DeriveInput,
    // Field,
    Fields,
    // FieldsNamed, FieldsUnnamed,
    Generics, Ident, Path, PathArguments, PathSegment, PredicateType,
    Type, TypeParam,
    TypeParamBound, TraitBound, TraitBoundModifier,
    // Visibility,
    WhereClause, WherePredicate,
};
use syn::punctuated::Punctuated;
// use syn::token::{Add, Colon, Colon2, Comma};

#[proc_macro_derive(Delta)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output: TokenStream2 = derive_internal(input).unwrap(
        // This is a HACK that allows more ergonomic code for the meat of
        // the macro while still conforming to the required macro signature.
    );
    TokenStream::from(output)
}

fn derive_internal(input: DeriveInput) -> DeriveResult<TokenStream2> {
    // let attrs: Vec<Attribute> = input.attrs;
    // let vis: Visibility = input.vis;
    let struct_name: Ident = input.ident;
    let mut generics: Generics = input.generics;
    let data: Data = input.data;

    #[derive(Debug)]
    enum FieldName {
        Struct(TokenTree2),
        Tuple(usize),
    }

    impl quote::ToTokens for FieldName {
        fn to_tokens(&self, tokens: &mut TokenStream2) {
            match self {
                Self::Struct(tt) =>
                    tokens.extend(TokenStream2::from(tt.clone())),
                Self::Tuple(num) => {
                    let lit = Literal2::usize_unsuffixed(*num);
                    tokens.extend(TokenStream2::from(TokenTree2::Literal(lit)));
                }
            }
        }
    }

    let mut field_idents: Vec<FieldName> = vec![];
    let mut field_props: Vec<Literal2> = vec![];
    let mut field_types: Vec<Type> = vec![];
    let mut numbered_field_types = vec![];
    let mut named_field_types = vec![];
    match data {
        Data::Union(_) =>
            unimplemented!("Computing Deltas of 2 unions is not supported."),
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named_fields) => { // structs with named field(s)
                for field in named_fields.named.iter() {
                    let field_ident: &Ident = field.ident.as_ref()
                        .expect("Found a bug in Rust itself");
                    let ident = TokenTree2::Ident(field_ident.clone());
                    let prop = Literal2::string(&format!("{}", field_ident));
                    field_idents.push(FieldName::Struct(ident));
                    field_props.push(prop);
                    field_types.push(field.ty.clone());
                    named_field_types.push(field.ty.clone());
                }
            },
            Fields::Unnamed(unnamed_fields) => { // Tuple structs
                for (fidx, field) in unnamed_fields.unnamed.iter().enumerate() {
                    field_idents.push(FieldName::Tuple(fidx));
                    field_props.push(Literal2::usize_unsuffixed(fidx));
                    field_types.push(field.ty.clone());
                    numbered_field_types.push(field.ty.clone());
                }
            },
            Fields::Unit => { // structs without fields
                // A unit struct has no fields, so it can supports `Delta`s.
            },
        },
        Data::Enum(DataEnum { variants, .. }) => {
            // TODO
            todo!()
        },
    }

    let type_param_decls = &generics.params;
    let type_params: TokenStream2 = {
        let comma = TokenTree2::Punct(Punct2::new(',', Spacing2::Alone));
        TokenStream2::from_iter(intersperse(comma, generics.type_params()))
    };

    if generics.where_clause.is_none() {
        // NOTE: initialize the `WhereClause` if there isn't one yet
        generics.where_clause = Some(WhereClause {
            where_token: Token![where](Span2::call_site()),
            predicates: Punctuated::new(),
        });
    }
    let trait_bound = |path_segments: &[&str]| TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path {
            leading_colon: None,
            segments: {
                let mut punct = Punctuated::new();
                for segment in path_segments {
                    punct.push(PathSegment {
                        arguments: PathArguments::None,
                        ident: Ident::new(segment, Span2::call_site()),
                    });
                }
                punct
            }
        },
    });
    if let Some(ref mut clause) = generics.where_clause.as_mut() {
        // NOTE: Add a clause for each field `f: F` of the form
        //    `F: struct_delta_trait::Delta + serde::Serialize`
        for field_type in field_types.iter() {
            clause.predicates.push(WherePredicate::Type(PredicateType {
                lifetimes: None,
                bounded_ty: field_type.clone(),
                colon_token: Token![:](Span2::call_site()),
                bounds: {
                    let mut bounds = Punctuated::new();
                    // Add `struct_delta_trait::Delta` as a type param bound:
                    bounds.push(trait_bound(&["struct_delta_trait", "DeltaOps"]));
                    // Add `serde::Serialize` as a type param bound:
                    bounds.push(trait_bound(&["serde", "Serialize"]));
                    bounds.push(trait_bound(&["PartialEq"]));
                    bounds
                },
            }));
        }
    }

    let where_clause: Option<&WhereClause> = generics.where_clause.as_ref();
    // let module_name = format_ident!("_delta_for_{}", struct_name);
    let delta_type_name = format_ident!("{}Delta", struct_name);


    let delta_struct_def = if !named_field_types.is_empty() { // regular struct
        quote! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct #delta_type_name<#type_param_decls> #where_clause {
                #(
                    pub(self) #field_idents: Option<#field_types>,
                )*
            }
        }
    } else if !numbered_field_types.is_empty() { // tuple struct
        quote! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct #delta_type_name<#type_param_decls> (
                #(
                    pub(self) Option<#field_types>,
                )*
            ) #where_clause ;
        }
    } else { // empty struct
        quote! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct #delta_type_name<#type_param_decls>
            #where_clause ;
        }
    };

    let output: TokenStream2 = quote! {
        // mod #module_name {
        #delta_struct_def
        // }

        impl<#type_param_decls>  struct_delta_trait::DeltaOps
            for  #struct_name<#type_params>  #where_clause
        {
            // type Delta = #module_name::#delta_type_name<#type_params>;
            type Delta = #delta_type_name<#type_params>;

            fn apply_delta(
                &self,
                delta: &Self::Delta,
            ) -> struct_delta_trait::DeltaResult<Self> {
                Ok(Self {
                    #( #field_idents: if let Some(fi) = &delta.#field_idents {
                        // This field appears in the delta, so
                        // the new instance should reflect that.
                        fi.clone()
                    } else {
                        // This field is not in the delta, so the value
                        // in the new instance is the value in `self`.
                        self.#field_idents.clone()
                    }),*
                })
            }

            fn delta(
                &self,
                rhs: &Self
            ) -> struct_delta_trait::DeltaResult<Self::Delta>
            {
                Ok(Self::Delta {
                    #( #field_idents: if self.#field_idents != rhs.#field_idents {
                        Some(rhs.#field_idents.clone())
                    } else {
                        None
                    } ),*
                })
            }

        }
    };

    Ok(output)
}


/// Intersperse a `token` in between the items in `iter`.
/// Return the resulting iterator.
fn intersperse<'i>(
    token: TokenTree2,
    iter: impl Iterator<Item = &'i TypeParam>,
) -> impl Iterator<Item = TokenTree2> {
    let mut type_params: Vec<TokenTree2> = vec![];
    let params = iter
        .map(|type_param| type_param.ident.clone())
        .map(TokenTree2::from);
    for (i, tp) in params.enumerate() {
        if i > 0 { type_params.push(token.clone()); }
        type_params.push(tp);
    }
    type_params.into_iter()

}



// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
