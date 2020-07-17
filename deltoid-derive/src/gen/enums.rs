//! Code generation for enums
#![allow(non_snake_case)]


use crate::DeriveResult;
use crate::gen::{EnumVariant, FieldDesc, InputType, StructVariant};
use itertools::iproduct;
use proc_macro2::{
    Ident as Ident2, Literal as Literal2, TokenStream as TokenStream2
};
use syn::*;
use quote::{format_ident, quote};

pub(crate) fn define_delta_enum(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        delta_type_name,
        enum_variants,
        type_param_decls: in_type_param_decls,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: trait bounds on the corresponding type parameter
                    //       `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T: deltoid::Core
                            // NOTE: don't include serde::{Deserialize, Serde}
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let where_clause = quote! { /*where*/ };
        let enum_body: TokenStream2 = enum_variants.iter()
            .map(|enum_variant: &EnumVariant| -> DeriveResult<_> {
                let variant_name = &enum_variant.name;
                let field_types: Vec<TokenStream2> = enum_variant.fields()
                    .map(|field: &FieldDesc| field.type_tokens())
                    .collect();
                Ok(match enum_variant.struct_variant {
                    StructVariant::NamedStruct => {
                        let field_names: Vec<&Ident2> = enum_variant.fields()
                            .map(|field: &FieldDesc| field.name_ref())
                            .collect::<DeriveResult<_>>()?;
                        quote! {
                            #variant_name {
                                #(
                                    #[doc(hidden)] #field_names: #field_types,
                                )*
                            },
                        }
                    },
                    StructVariant::TupleStruct => quote! {
                        #variant_name( #( #[doc(hidden)] #field_types, )* ),
                    },
                    StructVariant::UnitStruct => quote! {
                        #variant_name,
                    },
                })
            })
            .collect::<DeriveResult<_>>()?;
        Ok(quote! {
            #[derive(Clone, PartialEq)]
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub enum #delta_type_name<#(#type_param_decls),*>
                #where_clause
            {
                #enum_body
            }
        })
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Debug_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        type_name,
        delta_type_name,
        enum_variants: variants,
        type_param_decls: in_type_param_decls,
        type_params,
        where_clause: in_where_clause,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: `bounds` defines trait bounds on the corresponding
                    // type parameter `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T: deltoid::Core
                            + std::fmt::Debug
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let predicates: Vec<TokenStream2> = in_where_clause.predicates.iter()
            .map(|where_predicate| quote! { #where_predicate })
            .collect();
        let where_clause = quote! { where #(#predicates),* };
        let mut field_patterns: Vec<TokenStream2> = vec![];
        let mut match_bodies: Vec<TokenStream2> = vec![];
        for v in variants.iter() { match (v.struct_variant, &v.name, &v.fields) {
            (StructVariant::NamedStruct, variant_name, variant_fields) => {
                let field_names: Vec<&Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .collect();
                let buf: Ident2 = format_ident!("buf");
                let fields: Vec<TokenStream2> = variant_fields.iter()
                    .map(|field| {
                        let fname = field.name_ref()?;
                        let ftype = field.type_ref();
                        Ok(if field.ignore_field() {
                            quote! {
                                // NOTE: format the PhantomData field itself
                                #buf.field(stringify!(#fname), &#fname);
                            }
                        } else {
                            quote! {
                                let fname: &'static str = stringify!(#fname);
                                if let Some(#fname) =  &#fname {
                                    // NOTE: don't format the `Some()` wrapper
                                    #buf.field(fname, &#fname);
                                } else {
                                    #buf.field(fname, &None as &Option<#ftype>);
                                }
                            }
                        })
                    })
                    .collect::<DeriveResult<_>>()?;
                field_patterns.push(quote! {
                    Self::#variant_name { #(#field_names),* }
                });
                match_bodies.push(quote! {{
                    let type_name = String::new()
                        + stringify!(#delta_type_name)
                        + "::"
                        + stringify!(#variant_name);
                    let mut #buf = f.debug_struct(&type_name);
                    #( #fields )*
                    #buf.finish()
                }});
            },
            (StructVariant::TupleStruct, variant_name, variant_fields) => {
                let field_types: Vec<&Type> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.type_ref())
                    .collect();
                let field_count = field_types.len();
                let field_names: Vec<Ident2> = (0 .. field_count)
                    .map(|ident| format_ident!("field_{}", ident))
                    .collect();
                let buf: Ident2 = format_ident!("buf");
                let fields: Vec<TokenStream2> = variant_fields.iter()
                    .enumerate()
                    .map(|(fidx, field)| {
                        let fname = format_ident!("field_{}", fidx);
                        let ftype = field.type_ref();
                        Ok(match field_count {
                            1 => quote! { /* NOTE: the input is a newtype; NOP */ },
                            _ if field.ignore_field() => quote! {
                                // NOTE: the input is a regular tuple struct
                                // NOTE: format the PhantomData field itself
                                #buf.field(&#fname);
                            },
                            _ => quote! {
                                // NOTE: the input is a regular tuple struct
                                if let Some(field) = &#fname {
                                    // NOTE: don't format the `Some()` wrapper
                                    #buf.field(&field);
                                } else {
                                    #buf.field(&None as &Option<#ftype>);
                                }
                            },
                        })
                    })
                    .collect::<DeriveResult<_>>()?;
                field_patterns.push(quote! {
                    Self::#variant_name( #(#field_names),* )
                });
                match_bodies.push(match field_count {
                    1 => quote! {{
                        // NOTE: the input type is a newtype
                        let type_name = String::new()
                            + stringify!(#delta_type_name)
                            + "::"
                            + stringify!(#variant_name);
                        write!(f, "{}({:?})", type_name, field_0)
                    }},
                    _ => quote! {{
                        let type_name = String::new()
                            + stringify!(#delta_type_name)
                            + "::"
                            + stringify!(#variant_name);
                        let mut #buf = f.debug_tuple(&type_name);
                        #( #fields )*
                        #buf.finish()
                    }},
                });
            },
            (StructVariant::UnitStruct, variant_name, variant_fields) => {
                field_patterns.push(quote! {
                    Self::#variant_name
                });
                match_bodies.push(quote! {{
                    let type_name = String::new()
                        + stringify!(#delta_type_name)
                        + "::"
                        + stringify!(#variant_name);
                    f.debug_struct(&type_name).finish()
                }});
            },
        }}
        let body = quote! {
            match self {
                #(
                    #field_patterns => #match_bodies,
                )*
            }
        };
        Ok(quote! {
            impl<#(#type_param_decls),*> std::fmt::Debug
                for #delta_type_name<#type_params>
                #where_clause
            {
                fn fmt(&self, f: &mut std::fmt::Formatter)
                       -> Result<(), std::fmt::Error>
                {
                    #body
                }
            }
        })
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Core_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        type_name,
        delta_type_name,
        enum_variants,
        type_param_decls: in_type_param_decls,
        type_params,
        where_clause: in_where_clause,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: `bounds` defines trait bounds on the corresponding
                    // type parameter `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T: std::clone::Clone
                            + std::fmt::Debug
                            + std::cmp::PartialEq
                            + deltoid::Core
                            + for<'de> serde::Deserialize<'de>
                            + serde::Serialize
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let predicates: Vec<TokenStream2> = in_where_clause.predicates.iter()
            .map(|where_predicate| quote! { #where_predicate })
            .collect();
        let where_clause = quote! { where #(#predicates),* };
        Ok(quote! {
            impl<#(#type_param_decls),*> deltoid::Core
                for #type_name<#type_params>
                #where_clause
            {
                type Delta = #delta_type_name<#type_params>;
            }
        })
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Apply_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        type_name,
        delta_type_name,
        enum_variants: variants,
        type_param_decls: in_type_param_decls,
        type_params,
        where_clause: in_where_clause,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: `bounds` defines trait bounds on the corresponding
                    // type parameter `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T: std::clone::Clone
                            + std::fmt::Debug
                            + std::cmp::PartialEq
                            + deltoid::Apply
                            + deltoid::FromDelta
                            + for<'de> serde::Deserialize<'de>
                            + serde::Serialize
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let predicates: Vec<TokenStream2> = in_where_clause.predicates.iter()
            .map(|where_predicate| quote! { #where_predicate })
            .collect();
        let where_clause = quote! { where #(#predicates),* };
        let mut   lhs_patterns: Vec<TokenStream2> = vec![];
        let mut delta_patterns: Vec<TokenStream2> = vec![];
        let mut match_bodies: Vec<TokenStream2> = vec![];
        for v in variants.iter() { match (v.struct_variant, &v.name, &v.fields) {
            (StructVariant::NamedStruct, variant_name, variant_fields) => {
                let field_names: Vec<&Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .collect();
                let lhs_names: Vec<Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .map(|ident: &Ident2| format_ident!("lhs_{}", ident))
                    .collect();
                let delta_names: Vec<Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .map(|ident| format_ident!("delta_{}", ident))
                    .collect();
                let field_values: Vec<TokenStream2> = variant_fields.iter()
                    .zip(lhs_names.iter())
                    .zip(delta_names.iter())
                    .map(|((f, lhs_name), delta_name)| if f.ignore_field() {
                        quote! { #lhs_name.clone() }
                    } else {
                        quote! {
                            if let Some(delta) = #delta_name {
                                #lhs_name.apply(delta.clone(/*TODO*/))?
                            } else {
                                #lhs_name.clone()
                            }
                        }
                    })
                    .collect();
                // NOTE: first, push the pairwise-equal patterns:
                lhs_patterns.push(quote! {
                    Self::#variant_name {
                        #(#field_names: #lhs_names),*
                    }
                });
                delta_patterns.push(quote! {
                    Self::Delta::#variant_name {
                        #(#field_names: #delta_names),*
                    }
                });
                match_bodies.push(quote! {
                    Ok(Self::#variant_name {
                        #(#field_names: #field_values),*
                    })
                });
                // NOTE: then, push the pairwise-unequal patterns:
                lhs_patterns.push(quote! { _ });
                delta_patterns.push(quote! {
                    delta @ Self::Delta::#variant_name { .. }
                });
                match_bodies.push(quote! {
                    use deltoid::FromDelta;
                    Self::from_delta(delta.clone(/*TODO*/))
                });
            },
            (StructVariant::TupleStruct, variant_name, variant_fields) => {
                let field_types: Vec<&Type> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.type_ref())
                    .collect();
                let field_count = field_types.len();
                let lhs_names: Vec<Ident2> = (0 .. field_count)
                    .map(|ident| format_ident!("lhs_{}", ident))
                    .collect();
                let delta_names: Vec<Ident2> = (0 .. field_count)
                    .map(|ident| format_ident!("delta_{}", ident))
                    .collect();
                let field_values: Vec<TokenStream2> = variant_fields.iter()
                    .zip(lhs_names.iter())
                    .zip(delta_names.iter())
                    .map(|((f, lhs_name), delta_name)| if f.ignore_field() {
                        quote! { #lhs_name.clone() }
                    } else {
                        quote! {
                            if let Some(delta) = #delta_name {
                                #lhs_name.apply(delta.clone(/*TODO*/))?
                            } else {
                                #lhs_name.clone()
                            }
                        }
                    })
                    .collect();
                // NOTE: first, push the pairwise-equal patterns:
                lhs_patterns.push(quote! {
                    Self::#variant_name( #(#lhs_names),* )
                });
                delta_patterns.push(quote! {
                    Self::Delta::#variant_name( #(#delta_names),* )
                });
                match_bodies.push(quote! {
                    Ok(Self::#variant_name( #(#field_values),* ))
                });
                // NOTE: then, push the pairwise-unequal patterns:
                lhs_patterns.push(quote! { _ });
                delta_patterns.push(quote! {
                    delta @ Self::Delta::#variant_name(..)
                });
                match_bodies.push(quote! {
                    use deltoid::FromDelta;
                    Self::from_delta(delta.clone(/*TODO*/))
                });
            },
            (StructVariant::UnitStruct, variant_name, variant_fields) => {
                // NOTE: first, push the pairwise-equal patterns:
                lhs_patterns.push(quote! { Self::#variant_name });
                delta_patterns.push(quote! { Self::Delta::#variant_name });
                match_bodies.push(quote! { Ok(Self::#variant_name) });
                // NOTE: then, push the pairwise-unequal patterns:
                lhs_patterns.push(quote! { _ });
                delta_patterns.push(quote! { delta @ Self::Delta::#variant_name });
                match_bodies.push(quote! {
                    use deltoid::FromDelta;
                    Self::from_delta(delta.clone(/*TODO*/))
                });
            },
        }}
        Ok(quote! {
            impl<#(#type_param_decls),*> deltoid::Apply
                for #type_name<#type_params>
                #where_clause
            {
                #[allow(unused)]
                fn apply(&self, delta: Self::Delta) -> deltoid::DeltaResult<Self> {
                    match (self, &delta/*TODO*/) {
                        #(
                            (#lhs_patterns, #delta_patterns) => {
                                #match_bodies
                            },
                        )*
                    }
                }
            }
        })
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Delta_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        type_name,
        delta_type_name,
        enum_variants: variants,
        type_param_decls: in_type_param_decls,
        type_params,
        where_clause: in_where_clause,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: `bounds` defines trait bounds on the corresponding
                    // type parameter `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T: std::clone::Clone
                            + std::fmt::Debug
                            + std::cmp::PartialEq
                            + deltoid::Delta
                            + deltoid::IntoDelta
                            + for<'de> serde::Deserialize<'de>
                            + serde::Serialize
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let predicates: Vec<TokenStream2> = in_where_clause.predicates.iter()
            .map(|where_predicate| quote! { #where_predicate })
            .collect();
        let where_clause = quote! { where #(#predicates),* };
        let mut lhs_patterns: Vec<TokenStream2> = vec![];
        let mut rhs_patterns: Vec<TokenStream2> = vec![];
        let mut match_bodies: Vec<TokenStream2> = vec![];
        for v in variants.iter() { match (v.struct_variant, &v.name, &v.fields) {
            (StructVariant::NamedStruct, variant_name, variant_fields) => {
                let field_names: Vec<&Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .collect();
                let lhs_names: Vec<Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .map(|ident: &Ident2| format_ident!("lhs_{}", ident))
                    .collect();
                let rhs_names: Vec<Ident2> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .map(|ident| format_ident!("rhs_{}", ident))
                    .collect();
                let field_values: Vec<TokenStream2> = variant_fields.iter()
                    .zip(lhs_names.iter())
                    .zip(rhs_names.iter())
                    .map(|((f, lhs_name), rhs_name)| if f.ignore_field() {
                        quote! { std::marker::PhantomData }
                    } else {
                        quote! {
                            if #lhs_name == #rhs_name {
                                None
                            } else {
                                Some(#lhs_name.delta(#rhs_name)?)
                            }
                        }
                    })
                    .collect();
                // NOTE: first, push the pairwise-equal patterns:
                lhs_patterns.push(quote! {
                    Self::#variant_name { #(#field_names: #lhs_names),* }
                });
                rhs_patterns.push(quote! {
                    Self::#variant_name { #(#field_names: #rhs_names),* }
                });
                match_bodies.push(quote! {
                    Ok(Self::Delta::#variant_name {
                        #(#field_names: #field_values),*
                    })
                });
                // NOTE: then, push the pairwise-unequal patterns:
                lhs_patterns.push(quote! { _ });
                rhs_patterns.push(quote! { rhs @ Self::#variant_name { .. } });
                match_bodies.push(quote! {
                    use deltoid::IntoDelta;
                    rhs.clone().into_delta()
                });
            },
            (StructVariant::TupleStruct, variant_name, variant_fields) => {
                let field_types: Vec<&Type> = variant_fields.iter()
                    .map(|field: &FieldDesc| field.type_ref())
                    .collect();
                let field_count = field_types.len();
                let lhs_names: Vec<Ident2> = (0 .. field_count)
                    .map(|ident| format_ident!("lhs_{}", ident))
                    .collect();
                let rhs_names: Vec<Ident2> = (0 .. field_count)
                    .map(|ident| format_ident!("rhs_{}", ident))
                    .collect();
                let field_values: Vec<TokenStream2> = variant_fields.iter()
                    .zip(lhs_names.iter().zip(rhs_names.iter()))
                    .map(|(f, (lhs_name, rhs_name))| if f.ignore_field() {
                        quote! { std::marker::PhantomData }
                    } else {
                        quote! {
                            if #lhs_name == #rhs_name {
                                None
                            } else {
                                Some(#lhs_name.delta(#rhs_name)?)
                            }
                        }
                    })
                    .collect();
                // NOTE: first, push the pairwise-equal patterns:
                lhs_patterns.push(quote! {
                    Self::#variant_name( #(#lhs_names),* )
                });
                rhs_patterns.push(quote! {
                    Self::#variant_name( #(#rhs_names),* )
                });
                match_bodies.push(quote! {
                    Ok(Self::Delta::#variant_name( #(#field_values),* ))
                });
                // NOTE: then, push the pairwise-unequal patterns:
                lhs_patterns.push(quote! { _ });
                rhs_patterns.push(quote! { rhs @ Self::#variant_name(..) });
                match_bodies.push(quote! {
                    use deltoid::IntoDelta;
                    rhs.clone().into_delta()
                });
            },
            (StructVariant::UnitStruct, variant_name, variant_fields) => {
                // NOTE: first, push the pairwise-equal patterns:
                lhs_patterns.push(quote! { Self::#variant_name });
                rhs_patterns.push(quote! { Self::#variant_name });
                match_bodies.push(quote! { Ok(Self::Delta::#variant_name) });
                // NOTE: then, push the pairwise-unequal patterns:
                lhs_patterns.push(quote! { _ });
                rhs_patterns.push(quote! { rhs @ Self::#variant_name });
                match_bodies.push(quote! {
                    use deltoid::IntoDelta;
                    rhs.clone().into_delta()
                });
            },
        }}
        Ok(quote! {
            impl<#(#type_param_decls),*> deltoid::Delta
                for #type_name<#type_params>
                #where_clause
            {
                #[allow(unused)]
                fn delta(&self, rhs: &Self) -> deltoid::DeltaResult<Self::Delta> {
                    use deltoid::IntoDelta;
                    match (self, rhs) {
                        #(
                            (#lhs_patterns, #rhs_patterns) => { #match_bodies },
                        )*
                    }
                }
            }
        })
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_FromDelta_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        type_name,
        delta_type_name,
        enum_variants,
        type_params,
        type_param_decls: in_type_param_decls,
        where_clause: in_where_clause,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: `bounds` defines trait bounds on the corresponding
                    // type parameter `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T:
                            std::clone::Clone
                            + std::fmt::Debug
                            + std::cmp::PartialEq
                            + deltoid::FromDelta
                            + for<'de> serde::Deserialize<'de>
                            + serde::Serialize
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let predicates: Vec<TokenStream2> = in_where_clause.predicates.iter()
            .map(|where_predicate| quote! { #where_predicate })
            .collect();
        let where_clause = quote! { where #(#predicates),* };
        let mut match_body = TokenStream2::new();
        for variant in enum_variants.iter() {
            let variant_name = &variant.name;
            match_body.extend(match variant.struct_variant {
                StructVariant::NamedStruct => {
                    let field_names: Vec<_> = variant.fields()
                        .map(|field: &FieldDesc| field.name_ref())
                        .collect::<DeriveResult<_>>()?;
                    let field_assignments: Vec<TokenStream2> = variant.fields()
                        .map(|field: &FieldDesc| {
                            let fname = field.name_ref()?;
                            let ftype = field.type_ref();
                            Ok(if field.ignore_field() {
                                quote! { #fname: Default::default() }
                            } else {
                                quote! {
                                    #fname: <#ftype>::from_delta(
                                        #fname.ok_or_else(|| DeltaError::ExpectedValue {
                                            type_name: stringify!(#ftype).to_string(),
                                            file: file!().to_string(),
                                            line: line!(),
                                            column: column!(),
                                        })?
                                    )?
                                }
                            })
                        })
                        .collect::<DeriveResult<_>>()?;
                    quote! {
                        #delta_type_name::#variant_name { #(#field_names),* } => {
                            Self::#variant_name { #(#field_assignments),* }
                        },
                    }
                },
                StructVariant::TupleStruct => {
                    let field_types: Vec<_> = variant.fields()
                        .map(|field: &FieldDesc| field.type_ref())
                        .collect();
                    let field_count = field_types.len();
                    let field_names: Vec<Ident> = (0 .. field_count)
                        .map(|token| format_ident!("field{}", token))
                        .collect();
                    let field_assignments: Vec<TokenStream2> = variant.fields()
                        .enumerate()
                        .map(|(fidx, field): (usize, &FieldDesc)| {
                            let fname = &field_names[fidx];
                            let ftype = field.type_ref();
                            Ok(if field.ignore_field() {
                                quote! { Default::default() }
                            } else {
                                quote! {
                                    <#ftype>::from_delta(
                                        #fname.ok_or_else(|| DeltaError::ExpectedValue {
                                            type_name: stringify!(#ftype).to_string(),
                                            file: file!().to_string(),
                                            line: line!(),
                                            column: column!(),
                                        })?
                                    )?
                                }
                            })
                        })
                        .collect::<DeriveResult<_>>()?;
                    quote! {
                        #delta_type_name::#variant_name( #(#field_names),* ) => {
                            Self::#variant_name( #(#field_assignments),* )
                        },
                    }
                },
                StructVariant::UnitStruct => quote! {
                    #delta_type_name::#variant_name => {
                        Self::#variant_name
                    },
                },
            });
        }
        Ok(quote! {
            impl<#(#type_param_decls),*> deltoid::FromDelta
                for #type_name<#type_params>
                #where_clause
            {
                #[allow(unused)]
                fn from_delta(delta: Self::Delta) -> deltoid::DeltaResult<Self> {
                    #[allow(unused)] use deltoid::{DeltaError, FromDelta};
                    Ok(match delta {
                        #match_body
                    })
                }
            }
        })
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_IntoDelta_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Enum {
        type_name,
        delta_type_name,
        enum_variants,
        type_params,
        type_param_decls: in_type_param_decls,
        where_clause: in_where_clause,
        ..
    } = input {
        let type_param_decls: Vec<TokenStream2> = in_type_param_decls.iter()
            .map(|type_param_decl| match type_param_decl {
                GenericParam::Lifetime(lifetime_def) => quote! { #lifetime_def },
                GenericParam::Const(const_param)     => quote! { #const_param  },
                GenericParam::Type(type_param) => {
                    let T: &Ident2 = &type_param.ident;
                    // NOTE: `bounds` defines trait bounds on the corresponding
                    // type parameter `T` in `InputType::Struct#type_param`:
                    let bounds: Vec<TokenStream2> = type_param.bounds.iter()
                        .map(|trait_bound| quote! { #trait_bound })
                        .collect();
                    quote! {
                        #T:
                            std::clone::Clone
                            + std::fmt::Debug
                            + std::cmp::PartialEq
                            + deltoid::IntoDelta
                            + for<'de> serde::Deserialize<'de>
                            + serde::Serialize
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let predicates: Vec<TokenStream2> = in_where_clause.predicates.iter()
            .map(|where_predicate| quote! { #where_predicate })
            .collect();
        let where_clause = quote! { where #(#predicates),* };
        let mut match_body = TokenStream2::new();
        for enum_variant in enum_variants.iter() {
            let struct_variant = enum_variant.struct_variant;
            let variant_name = &enum_variant.name;
            match_body.extend(match struct_variant {
                StructVariant::NamedStruct => {{
                    let field_names: Vec<_> = enum_variant.fields()
                        .map(|field: &FieldDesc| field.name_ref())
                        .collect::<DeriveResult<_>>()?;
                    let field_assignments: Vec<TokenStream2> = enum_variant.fields()
                        .map(|field: &FieldDesc| {
                            let fname = field.name_ref()?;
                            Ok(if field.ignore_field() {
                                quote! { #fname: std::marker::PhantomData }
                            } else {
                                quote! { #fname: Some(#fname.into_delta()?) }
                            })
                        })
                        .collect::<DeriveResult<_>>()?;
                    quote! {
                        Self::#variant_name { #(#field_names),* } => {
                            #delta_type_name::#variant_name {
                                #(#field_assignments),*
                            }
                        },
                    }
                }},
                StructVariant::TupleStruct => {{
                    let field_count = enum_variant.fields().count();
                    let field_names: Vec<Ident> = (0 .. field_count)
                        .map(|token| format_ident!("field_{}", token))
                        .collect();
                    let field_assignments: Vec<TokenStream2> = enum_variant.fields()
                        .enumerate()
                        .map(|(fidx, field): (usize, &FieldDesc)| {
                            let fname = &field_names[fidx];
                            Ok(if field.ignore_field() {
                                quote! { std::marker::PhantomData }
                            } else {
                                quote! { Some(#fname.into_delta()?) }
                            })
                        })
                        .collect::<DeriveResult<_>>()?;
                    quote! {
                        Self::#variant_name( #(#field_names),* ) => {
                            #delta_type_name::#variant_name(
                                #(#field_assignments),*
                            )
                        },
                    }
                }},
                StructVariant::UnitStruct => quote! {
                    Self::#variant_name => {
                        #delta_type_name::#variant_name
                    },
                },
            });
        }

        Ok(quote! {
            impl<#(#type_param_decls),*> deltoid::IntoDelta
                for #type_name<#type_params>
                #where_clause
            {
                #[allow(unused)]
                fn into_delta(self) -> deltoid::DeltaResult<Self::Delta> {
                    #[allow(unused)] use deltoid::{DeltaError, IntoDelta};
                    Ok(match self {
                        #match_body
                    })
                }
            }
        })
    } else {
        bug_detected!()
    }
}
