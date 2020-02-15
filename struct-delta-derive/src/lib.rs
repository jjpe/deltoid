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
    Attribute,
    Data, DataStruct, DataEnum, DeriveInput,
    // Field,
    Expr, Fields,
    // FieldsNamed, FieldsUnnamed,
    GenericParam, Generics, Ident, Path, PathArguments, PathSegment, PredicateType,
    Type, TypeParam,
    TypeParamBound, TraitBound, TraitBoundModifier,
    Visibility,
    WhereClause, WherePredicate,
};
use syn::punctuated::Punctuated;
use syn::token::Eq;
use syn::token::{Add, Colon, Colon2, Comma};

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

    let mut field_idents: Vec<FieldName> = vec![];
    let mut field_props: Vec<Literal2> = vec![];
    let mut field_types: Vec<Type> = vec![];
    let mut numbered_field_types: Vec<Type> = vec![];
    let mut named_field_types: Vec<Type> = vec![];
    let mut struct_variant: Option<StructVariant> = None;
    let mut set_struct_variant = |variant| {
        if struct_variant.is_none() {
            Ok({ struct_variant = Some(variant); })
        } else if struct_variant == Some(variant) {
            Ok(())  // NOP
        } else {
            bug_detected!()
        }
    };

    match data {
        Data::Union(_) =>
            unimplemented!("Computing Deltas of 2 unions is not supported."),
        Data::Struct(DataStruct { fields, .. }) if fields.len() == 0 => {
            set_struct_variant(StructVariant::UnitStruct)?;
        },
        Data::Struct(DataStruct { fields, .. }) =>
            for (fidx, field) in fields.iter().enumerate() {
                if let Some(ident) = field.ident.as_ref() {
                    // A struct with named fields
                    let ident = TokenTree2::Ident(ident.clone());
                    field_props.push(Literal2::string(&format!("{}", ident)));
                    field_idents.push(FieldName::Struct(ident));
                    field_types.push(field.ty.clone());
                    named_field_types.push(field.ty.clone());
                    set_struct_variant(StructVariant::NamedStruct)?;
                } else { // A tuple struct i.e. unnamed/positional fields
                    field_props.push(Literal2::usize_unsuffixed(fidx));
                    field_idents.push(FieldName::Tuple(fidx));
                    field_types.push(field.ty.clone());
                    numbered_field_types.push(field.ty.clone());
                    set_struct_variant(StructVariant::TupleStruct)?;
                }
            },
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                let variant_attrs: &[Attribute] = &variant.attrs;
                let variant_ident: &Ident = &variant.ident;
                for field in variant.fields {
                    // let field_attrs: &[Attribute] = &field.attrs;
                    // let field_vis: &Visibility = &field.vis;
                    let field_ident: Option<&Ident> = field.ident.as_ref();
                    let field_type: &Type = &field.ty;

                    // TODO
                }
                // let variant_discriminant: Option<&(Eq, Expr)> =
                //     variant.discriminant.as_ref();
                if let Some((_eq, expr)) = variant.discriminant.as_ref() {
                    // TODO
                } else {
                    // TODO
                }
                // TODO
            }

            // TODO
            todo!()
        },
    }

    add_type_paramn_bounds_to_where_clause(&mut generics, &field_types);
    let where_clause: Option<&WhereClause> = generics.where_clause.as_ref();
    let type_param_decls = &generics.params;
    let type_params: Punctuated<TokenTree2, Comma> = generics.type_params()
        .map(|type_param| type_param.ident.clone())
        .map(TokenTree2::from)
        .collect();
    let delta_type_name = format_ident!("{}Delta", struct_name);
    let delta_struct_definition = define_delta_struct(
        struct_variant.unwrap(/*TODO Option*/),
        &delta_type_name,
        type_param_decls,
        where_clause,
        &field_idents,
        field_types
    );

    let output: TokenStream2 = quote! {
        #delta_struct_definition

        impl<#type_param_decls>  struct_delta_trait::DeltaOps
            for  #struct_name<#type_params>  #where_clause
        {
            type Delta = #delta_type_name<#type_params>;

            fn apply_delta(&self, delta: &Self::Delta)
                           -> struct_delta_trait::DeltaResult<Self>
            {
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

            fn delta(&self, rhs: &Self)
                     -> struct_delta_trait::DeltaResult<Self::Delta>
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



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructVariant { UnitStruct, TupleStruct, NamedStruct }

fn define_delta_struct(
    struct_variant: StructVariant,
    delta_type_name: &Ident,
    type_param_decls: &Punctuated<GenericParam, Comma>,
    where_clause: Option<&WhereClause>,
    // field_idents: &[Ident],
    field_idents: &[FieldName],
    field_types: Vec<Type>,
) -> TokenStream2 {
    match struct_variant {
        StructVariant::NamedStruct => quote! {
            #[derive(Debug, PartialEq)]
            pub struct #delta_type_name<#type_param_decls> #where_clause {
                #(
                    pub(self) #field_idents: Option<#field_types>,
                )*
            }
        },
        StructVariant::TupleStruct => quote! {
            #[derive(Debug, PartialEq)]
            pub struct #delta_type_name<#type_param_decls> (
                #(
                    pub(self) Option<#field_types>,
                )*
            ) #where_clause ;
        },
        StructVariant::UnitStruct => quote! {
            #[derive(Debug, PartialEq)]
            pub struct #delta_type_name<#type_param_decls>
                #where_clause ;
        },
    }
}

fn add_type_paramn_bounds_to_where_clause(
    generics: &mut Generics,
    field_types: &[Type],
) {
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
            segments: path_segments.iter()
                .map(|segment| PathSegment {
                    arguments: PathArguments::None,
                    ident: Ident::new(segment, Span2::call_site()),
                })
                .collect(),
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
                bounds: vec![ // Add type param bounds
                    trait_bound(&["struct_delta_trait", "DeltaOps"]),
                    trait_bound(&["serde", "Serialize"]),
                    trait_bound(&["PartialEq"])
                ].into_iter().collect(),
            }));
        }
    }
}

#[derive(Debug)]
enum FieldName {
    Struct(TokenTree2),
    Tuple(usize),
}

impl quote::ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(TokenStream2::from(match self {
            Self::Struct(tt) => tt.clone(),
            Self::Tuple(num) =>
                TokenTree2::Literal(Literal2::usize_unsuffixed(*num)),
        }));
    }
}
