//! Code generation for structs
#![allow(non_snake_case)]

use crate::DeriveResult;
use crate::gen::{FieldDesc, InputType, StructVariant};
use proc_macro2::{
    Ident as Ident2, Literal as Literal2, TokenStream as TokenStream2
};
use syn::*;
use syn::punctuated::Punctuated;
use syn::token::{Add, Comma};
use quote::{format_ident, quote};

pub(crate) fn define_delta_struct(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Struct {
        struct_variant,
        delta_type_name,
        fields,
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
                            #(+ #bounds)* // Copy user-specified type/lifetime bounds
                    }
                },
            })
            .collect();
        let field_types: Vec<TokenStream2> = fields.iter()
            .map(|field: &FieldDesc| field.type_tokens())
            .collect();
        let where_clause = quote! { where };
        match struct_variant {
            StructVariant::NamedStruct => {
                let field_names: Vec<&Ident2> = fields.iter()
                    .map(|field: &FieldDesc| field.name_ref())
                    .collect::<DeriveResult<_>>()?;
                Ok(quote! {
                    #[derive(Clone, PartialEq)]
                    #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
                    pub struct #delta_type_name<#(#type_param_decls),*>
                        #where_clause
                    {
                        #( #[doc(hidden)] pub(self) #field_names: #field_types, )*
                    }

                    // TODO: Add a {Eq, Hash} impl for `#delta_type_name`
                    // where `T: {Eq, Hash}` for every generic type arg `T`.
                })
            },
            StructVariant::TupleStruct => Ok(quote! {
                #[derive(Clone, PartialEq)]
                #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
                pub struct #delta_type_name<#(#type_param_decls),*> (
                    #( #[doc(hidden)] pub(self) #field_types, )*
                ) #where_clause ;

                // TODO: Add a {Eq, Hash} impl for `#delta_type_name`
                // where `T: {Eq, Hash}` for every generic type arg `T`.
            }),
            StructVariant::UnitStruct => Ok(quote! {
                #[derive(Clone, PartialEq, Eq, Hash)]
                #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
                pub struct #delta_type_name<#(#type_param_decls),*>
                    #where_clause ;
            }),
        }
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Debug_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Struct {
        struct_variant,
        type_name,
        delta_type_name,
        fields,
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
        match struct_variant {
            StructVariant::NamedStruct => {
                let field_names: Vec<&Ident2> = fields.iter()
                    .map(|field: &FieldDesc| field.name_ref().unwrap())
                    .collect();
                let field_types: Vec<&Type> = fields.iter()
                    .map(|field: &FieldDesc| field.type_ref())
                    .collect();
                let mut body = TokenStream2::new();
                let buf: Ident2 = format_ident!("buf");
                for field in fields.iter() {
                    let (fname, ftype) = (field.name_ref()?, field.type_ref());
                    body.extend(if field.ignore_field() {
                        quote! {
                            // NOTE: format the PhantomData field itself
                            #buf.field(stringify!(#fname), &self.#fname);
                        }
                    } else {
                        quote! {
                            let fname: &'static str = stringify!(#fname);
                            if let Some(#fname) =  &self.#fname {
                                // NOTE: don't format the `Some()` wrapper
                                #buf.field(fname, #fname);
                            } else {
                                #buf.field(fname, &None as &Option<#ftype>);
                            }
                        }
                    });
                }
                Ok(quote! {
                    impl<#(#type_param_decls),*> std::fmt::Debug
                        for #delta_type_name<#type_params>
                        #where_clause
                    {
                        fn fmt(&self, f: &mut std::fmt::Formatter)
                               -> Result<(), std::fmt::Error>
                        {
                            const NAME: &str = stringify!(#delta_type_name);
                            let mut #buf = f.debug_struct(NAME);
                            #body
                            #buf.finish()
                        }
                    }
                })
            },
            StructVariant::TupleStruct => {
                let field_types: Vec<&Type> = fields.iter()
                    .map(|field: &FieldDesc| field.type_ref())
                    .collect();
                let field_count = field_types.len();
                let field_names: Vec<Ident2> = (0 .. field_count)
                    .map(|ident| format_ident!("field_{}", ident))
                    .collect();
                let field_nums: Vec<Literal2> = (0 .. field_count)
                    .map(Literal2::usize_unsuffixed)
                    .collect();
                let mut field_tokens = TokenStream2::new();
                let buf: Ident2 = format_ident!("buf");
                for field in fields.iter() {
                    let (fpos, ftype) = (field.pos_ref()?, field.type_ref());
                    let fname = format_ident!("field");
                    field_tokens.extend(match field_count {
                        1 => quote! { /* NOTE: the input is a newtype; NOP*/ },
                        _ if field.ignore_field() => quote! {
                            // NOTE: the input is a regular tuple struct
                            // NOTE: format the PhantomData field itself
                            #buf.field(&self.#fpos);
                        },
                        _ => quote! {
                            // NOTE: the input is a regular tuple struct
                            if let Some(#fname) = &self.#fpos {
                                // NOTE: don't format the `Some()` wrapper
                                #buf.field(#fname);
                            } else {
                                #buf.field(&None as &Option<#ftype>);
                            }
                        },
                    });
                }
                let body = match field_count {
                    1 => quote! {
                        // NOTE: the input type is a newtype
                        const NAME: &str = stringify!(#delta_type_name);
                        if let Some(field) = &self.0 {
                            // NOTE: don't format the `Some()` wrapper
                            write!(f, "{}({:?})", NAME, field)
                        } else {
                            let field = &None as &Option<(/*HACK*/)>;
                            write!(f, "{}({:?})", NAME, field)
                        }
                    },
                    _ => quote! {
                        const NAME: &str = stringify!(#delta_type_name);
                        let mut #buf = f.debug_tuple(NAME);
                        #field_tokens
                        #buf.finish()
                    },
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
            },
            StructVariant::UnitStruct => {
                Ok(quote! {
                    impl<#(#type_param_decls),*> std::fmt::Debug
                        for #delta_type_name<#type_params>
                        #where_clause
                    {
                        fn fmt(&self, f: &mut std::fmt::Formatter)
                               -> Result<(), std::fmt::Error>
                        {
                            f.debug_struct(stringify!(#delta_type_name))
                                .finish()
                        }
                    }
                })
            },
        }
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Core_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Struct {
        struct_variant,
        type_name,
        delta_type_name,
        fields,
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
        match struct_variant {
            StructVariant::NamedStruct => Ok(quote! {
                impl<#(#type_param_decls),*> deltoid::Core
                    for #type_name<#type_params>
                    #where_clause
                {
                    type Delta = #delta_type_name<#type_params>;
                }
            }),
            StructVariant::TupleStruct => Ok(quote! {
                impl<#(#type_param_decls),*> deltoid::Core
                    for #type_name<#type_params>
                    #where_clause
                {
                    type Delta = #delta_type_name<#type_params>;
                }
            }),
            StructVariant::UnitStruct => Ok(quote! {
                impl<#(#type_param_decls),*> deltoid::Core
                    for #type_name<#type_params>
                    #where_clause
                {
                    type Delta = #delta_type_name<#type_params>;
                }
            }),
        }
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Apply_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Struct {
        struct_variant,
        type_name,
        delta_type_name,
        fields,
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
        match struct_variant {
            StructVariant::NamedStruct => {
                let field_assignments: Vec<TokenStream2> = fields.iter()
                    .map(|field: &FieldDesc| {
                        let fname = field.name_ref()?;
                        Ok(if field.ignore_field() {
                            quote! {
                                #fname: self.#fname.clone(),
                            }
                        } else {
                            quote! {
                                #fname: if let Some(d) = delta.#fname {
                                    self.#fname.apply(d)?
                                } else {
                                    self.#fname.clone()
                                },
                            }
                        })
                    })
                    .collect::<DeriveResult<_>>()?;
                Ok(quote! {
                    impl<#(#type_param_decls),*> deltoid::Apply
                        for #type_name<#type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn apply(&self, delta: Self::Delta)
                                 -> deltoid::DeltaResult<Self>
                        {
                            Ok(Self { #(#field_assignments)* })
                        }
                    }
                })
            },
            StructVariant::TupleStruct => {
                let field_assignments: Vec<TokenStream2> = fields.iter()
                    .map(|field: &FieldDesc| {
                        let fpos = field.pos_ref()?;
                        Ok(if field.ignore_field() {
                            quote! {
                                self.#fpos.clone(),
                            }
                        } else {
                            quote! {
                                if let Some(d) = delta.#fpos {
                                    self.#fpos.apply(d)?
                                } else {
                                    self.#fpos.clone()
                                },
                            }
                        })
                    })
                    .collect::<DeriveResult<_>>()?;
                Ok(quote! {
                    impl<#(#type_param_decls),*> deltoid::Apply
                        for #type_name<#type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn apply(&self, delta: Self::Delta)
                                 -> deltoid::DeltaResult<Self>
                        {
                            Ok(Self( #(#field_assignments)* ))
                        }
                    }
                })
            },
            StructVariant::UnitStruct => {
                Ok(quote! {
                    impl<#(#type_param_decls),*> deltoid::Apply
                        for #type_name<#type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn apply(&self, delta: Self::Delta)
                                 -> deltoid::DeltaResult<Self>
                        {
                            Ok(Self)
                        }
                    }
                })
            },
        }
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_Delta_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Struct {
        struct_variant,
        type_name,
        delta_type_name,
        fields,
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
                        #T:
                            std::clone::Clone
                            + std::fmt::Debug
                            + std::cmp::PartialEq
                            + deltoid::Delta
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
        match struct_variant {
            StructVariant::NamedStruct => {
                let field_assignments: Vec<TokenStream2> = fields.iter()
                    .map(|field: &FieldDesc| {
                        let fname = field.name_ref()?;
                        Ok(if field.ignore_field() {
                            quote! { #fname: std::marker::PhantomData }
                        } else {
                            quote! {
                                #fname: if self.#fname != rhs.#fname {
                                    Some(self.#fname.delta(&rhs.#fname)?)
                                } else {
                                    None
                                }
                            }
                        })
                    })
                    .collect::<DeriveResult<_>>()?;
                Ok(quote! {
                    impl<#(#type_param_decls),*> deltoid::Delta
                        for #type_name<#type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn delta(&self, rhs: &Self) ->
                            deltoid::DeltaResult<Self::Delta>
                        {
                            use deltoid::IntoDelta;
                            Ok(#delta_type_name { #(#field_assignments),* })
                        }
                    }
                })
            },
            StructVariant::TupleStruct => {
                let field_assignments: Vec<TokenStream2> = fields.iter()
                    .map(|field: &FieldDesc| {
                        let fpos = field.pos_ref()?;
                        Ok(if field.ignore_field() {
                            quote! { std::marker::PhantomData }
                        } else {
                            quote! {
                                if self.#fpos != rhs.#fpos {
                                    Some(self.#fpos.delta(&rhs.#fpos)?)
                                } else {
                                    None
                                }
                            }
                        })
                    })
                    .collect::<DeriveResult<_>>()?;
                Ok(quote! {
                    impl<#(#type_param_decls),*> deltoid::Delta
                        for #type_name<#type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn delta(&self,rhs: &Self) ->
                            deltoid::DeltaResult<Self::Delta>
                        {
                            use deltoid::IntoDelta;
                            Ok(#delta_type_name( #(#field_assignments),* ))
                        }
                    }
                })
            },
            StructVariant::UnitStruct => Ok(quote! {
                impl<#(#type_param_decls),*> deltoid::Delta
                    for #type_name<#type_params>
                    #where_clause
                {
                    #[allow(unused)]
                    fn delta(&self,rhs: &Self) ->
                        deltoid::DeltaResult<Self::Delta>
                    {
                        Ok(#delta_type_name)
                    }
                }
            }),
        }
    } else {
        bug_detected!()
    }
}

pub(crate) fn define_FromDelta_impl(input: &InputType) -> DeriveResult<TokenStream2> {
    if let InputType::Struct {
        struct_variant,
        type_name,
        delta_type_name,
        type_params,
        type_param_decls: in_type_param_decls,
        fields,
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
        let match_body: TokenStream2 = match struct_variant {
            StructVariant::NamedStruct => {
                let field_names: Vec<_> = fields.iter()
                    .map(|field: &FieldDesc| field.name_ref())
                    .collect::<DeriveResult<_>>()?;
                let field_assignments: Vec<TokenStream2> = fields.iter()
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
                    #delta_type_name { #(#field_names),* } => {
                        Self { #(#field_assignments),* }
                    },
                }
            },
            StructVariant::TupleStruct => {
                let field_types: Vec<_> = fields.iter()
                    .map(|field: &FieldDesc| field.type_ref())
                    .collect();
                let field_count = field_types.len();
                let field_names: Vec<Ident> = (0 .. field_count)
                    .map(|token| format_ident!("field{}", token))
                    .collect();
                let field_assignments: Vec<TokenStream2> = fields.iter()
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
                    #delta_type_name( #(#field_names),* ) => {
                        Self( #(#field_assignments),* )
                    },
                }
            },
            StructVariant::UnitStruct => quote! {
                #delta_type_name => Self,
            },
        };
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
    if let InputType::Struct {
        struct_variant,
        type_name,
        delta_type_name,
        type_params,
        type_param_decls: in_type_param_decls,
        fields,
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
        match_body.extend(match struct_variant {
            StructVariant::NamedStruct => {
                let field_names: Vec<_> = fields.iter()
                    .map(|field: &FieldDesc| field.name_ref())
                    .collect::<DeriveResult<_>>()?;
                let field_assignments: Vec<TokenStream2> = fields.iter()
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
                    Self { #( #field_names, )* .. } => {
                        #delta_type_name { #(#field_assignments),* }
                    },
                }
            },
            StructVariant::TupleStruct => {
                let field_count = fields.len();
                let field_names: Vec<Ident> = (0 .. field_count)
                    .map(|token| format_ident!("field_{}", token))
                    .collect();
                let field_assignments: Vec<TokenStream2> = fields.iter()
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
                    Self(#( #field_names, )* ..) => {
                        #delta_type_name( #(#field_assignments),* )
                    },
                }
            },
            StructVariant::UnitStruct => quote! {
                Self => #delta_type_name,
            },
        });
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
