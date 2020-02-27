extern crate proc_macro;

#[macro_use] mod error;

use itertools::{iproduct};
use crate::error::{DeriveResult};
use proc_macro::TokenStream;
use proc_macro2::{
    Literal as Literal2, Span as Span2, TokenStream as TokenStream2
};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Token/* macro that expands to a type, not a type */,
    AngleBracketedGenericArguments, BoundLifetimes,
    Data, DataStruct, DataEnum, DeriveInput, Expr,
    GenericArgument, GenericParam, Ident, Lifetime, LifetimeDef,
    Path, PathArguments, PathSegment, PredicateType,
    TraitBound, TraitBoundModifier, Type, TypeParamBound, TypePath,
    WhereClause, WherePredicate,
};
use syn::punctuated::Punctuated;
use syn::token::{Comma};


#[derive(Debug)]
struct EnumVariant {
    struct_variant: StructVariant,
    name: Ident,
    field_names: Vec<Ident>,
    field_types: Vec<Type>,
}

#[derive(Debug)]
struct Info {
    input_data_type: DataType,
    input_struct_variant: StructVariant,
    input_type_name: Ident,
    input_field_names: Vec<Ident>,
    input_field_types: Vec<Type>,
    input_type_param_decls: Punctuated<GenericParam, Comma>,
    input_type_params: Punctuated<Ident, Comma>,
    enum_variants: Vec<EnumVariant>,
    /// WhereClause for generated type definitions
    type_def_where_clause: WhereClause,
    /// WhereClause for the generated `DeltaOps` impl
    deltaops_trait_impl_where_clause: WhereClause,
    /// The name of the Delta type generated as a dual for the input type
    delta_type_name: Ident,
}


#[proc_macro_derive(Delta)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output: TokenStream2 = derive_internal(input).unwrap(
        // This is a HACK that allows more ergonomic code for the meat of
        // the macro while still conforming to the required macro signature.
    );
    TokenStream::from(output)
}

#[allow(non_snake_case)]
fn derive_internal(input: DeriveInput) -> DeriveResult<TokenStream2> {
    let mut info = Info {
        input_data_type: DataType::Struct,
        input_struct_variant: StructVariant::UnitStruct,
        input_type_name: input.ident.clone(),
        input_field_names: vec![],
        input_field_types: vec![],
        input_type_param_decls: input.generics.params.clone(),
        input_type_params: input.generics.type_params()
            .map(|type_param| type_param.ident.clone())
            .collect(),
        enum_variants: vec![],
        type_def_where_clause: input.generics.where_clause.as_ref().cloned()
            .unwrap_or_else(empty_where_clause),
        deltaops_trait_impl_where_clause: input.generics.where_clause
            .unwrap_or_else(empty_where_clause),
        delta_type_name: format_ident!("{}Delta", input.ident),
    };

    match input.data {
        Data::Struct(DataStruct { fields, .. }) if !fields.is_empty() => {
            info.input_data_type = DataType::Struct;
            for field in fields.iter() {
                info.input_field_types.push(field.ty.clone());
                if let Some(ident) = field.ident.as_ref() {
                    info.input_struct_variant = StructVariant::NamedStruct;
                    info.input_field_names.push(ident.clone());
                } else {
                    info.input_struct_variant = StructVariant::TupleStruct;
                }
            }
        },
        Data::Struct(DataStruct { .. }) => {
            info.input_data_type = DataType::Struct;
            info.input_struct_variant = StructVariant::UnitStruct;
        },
        Data::Enum(DataEnum { variants, .. }) => {
            info.input_data_type = DataType::Enum;
            for variant in variants {
                let mut enum_variant = EnumVariant {
                    struct_variant: StructVariant::UnitStruct,
                    name: variant.ident.clone(),
                    field_names: vec![],
                    field_types: vec![],
                };
                for field in variant.fields.iter() {
                    enum_variant.field_types.push(field.ty.clone());
                    if let Some(field_ident) = field.ident.as_ref() {
                        enum_variant.struct_variant = StructVariant::NamedStruct;
                        enum_variant.field_names.push(field_ident.clone());
                    } else {
                        enum_variant.struct_variant = StructVariant::TupleStruct;
                    }
                }
                info.enum_variants.push(enum_variant);

                // let variant_discriminant: Option<&(Eq, Expr)> =
                //     variant.discriminant.as_ref();
                if let Some((_eq, expr)) = variant.discriminant.as_ref() {
                    let _expr: &Expr = expr;
                    todo!("No support for enum variant discriminants yet");
                    // TODO
                } else {
                    // TODO
                }
            }
        },
        Data::Union(_) => { info.input_data_type = DataType::Union; },
    }

    // Enhance the struct/enum definition where clause
    match info.input_data_type {
        DataType::Struct => enhance_where_clause_for_type_def(
            &info.input_type_param_decls,
            &mut info.type_def_where_clause
        )?,
        DataType::Enum   => enhance_where_clause_for_type_def(
            &info.input_type_param_decls,
            &mut info.type_def_where_clause
        )?,
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
    }

    // Enhance the `impl DeltaOps for <<SOME_TYPE>>` where clause
    match info.input_data_type {
        DataType::Struct => enhance_where_clause_for_deltaops_trait_impl(
            &info.input_type_param_decls,
            &mut info.deltaops_trait_impl_where_clause
        )?,
        DataType::Enum   => enhance_where_clause_for_deltaops_trait_impl(
            &info.input_type_param_decls,
            &mut info.deltaops_trait_impl_where_clause
        )?,
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
    }

    let delta_type_definition: TokenStream2 = match info.input_data_type {
        DataType::Struct => define_delta_struct(
            info.input_struct_variant,
            &info.delta_type_name,
            &info.input_type_param_decls,
            &info.type_def_where_clause,
            &info.input_field_names,
            &info.input_field_types
        ),
        DataType::Enum => define_delta_enum(
            &info.delta_type_name,
            &info.input_type_param_decls,
            &info.type_def_where_clause,
            &info.enum_variants
        ),
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
    };

    let impl_DeltaOps_for_input_type  = impl_DeltaOps_for_input_type(&info);
    let impl_IntoDelta_for_input_type = impl_IntoDelta_for_input_type(&info);
    let impl_FromDelta_for_input_type = impl_FromDelta_for_input_type(&info);
    let output: TokenStream2 = quote! {
        #delta_type_definition

        #impl_DeltaOps_for_input_type
        #impl_IntoDelta_for_input_type
        #impl_FromDelta_for_input_type
    };

    // println!("{}", output);
    Ok(output)
}



#[allow(non_snake_case)]
fn impl_DeltaOps_for_input_type(info: &Info) -> TokenStream2 {
    match info.input_data_type {
        DataType::Struct => {
            let input_type_name = &info.input_type_name;
            let input_type_params = &info.input_type_params;
            let input_type_param_decls = &info.input_type_param_decls;
            let input_field_names = &info.input_field_names;
            // let input_field_types = &info.input_field_types;
            let where_clause = &info.deltaops_trait_impl_where_clause;
            let delta_type_name = &info.delta_type_name;
            match info.input_struct_variant {
                StructVariant::NamedStruct => quote! {
                    impl<#input_type_param_decls>  struct_delta_trait::DeltaOps
                        for  #input_type_name<#input_type_params>  #where_clause
                    {
                        type Delta = #delta_type_name<#input_type_params>;

                        fn apply_delta(
                            &self,
                            delta: &Self::Delta
                        ) -> struct_delta_trait::DeltaResult<Self> {
                            Ok(Self {
                                #(
                                    #input_field_names:
                                    if let Some(fdelta) = &delta.#input_field_names {
                                        self.#input_field_names.apply_delta(&fdelta)?
                                    } else {
                                        self.#input_field_names.clone()
                                    }
                                ),*
                            })
                        }

                        fn delta(
                            &self,
                            rhs: &Self
                        ) -> struct_delta_trait::DeltaResult<Self::Delta> {
                            Ok(Self::Delta {
                                #(
                                    #input_field_names:
                                    if self.#input_field_names != rhs.#input_field_names {
                                        Some(self.#input_field_names.delta(
                                            &rhs.#input_field_names
                                        )?)
                                    } else {
                                        None
                                    }
                                ),*
                            })
                        }
                    }
                },
                StructVariant::TupleStruct => {
                    let fields: Vec<_> = (0 .. info.input_field_types.len())
                        .map(Literal2::usize_unsuffixed)
                        .collect();
                    let delta_type_name = &info.delta_type_name;
                    quote! {
                        impl<#input_type_param_decls>  struct_delta_trait::DeltaOps
                            for  #input_type_name<#input_type_params>
                            #where_clause
                        {
                            type Delta = #delta_type_name<#input_type_params>;

                            fn apply_delta(
                                &self,
                                delta: &Self::Delta
                            ) -> struct_delta_trait::DeltaResult<Self> {
                                Ok(Self(
                                    #(
                                        if let Some(d) = &delta.#fields {
                                            self.#fields.apply_delta(&d)?
                                        } else {
                                            self.#fields.clone()
                                        }
                                    ),*
                                ))
                            }

                            fn delta(
                                &self,
                                rhs: &Self
                            ) -> struct_delta_trait::DeltaResult<Self::Delta> {
                                Ok(#delta_type_name(
                                    #(
                                        if self.#fields != rhs.#fields {
                                            Some(self.#fields.delta(&rhs.#fields)?)
                                        } else {
                                            None
                                        }
                                    ),*
                                ))
                            }
                        }
                    }
                },
                StructVariant::UnitStruct => {
                    let delta_type_name = &info.delta_type_name;
                    quote! {
                        impl<#input_type_param_decls>  struct_delta_trait::DeltaOps
                            for  #input_type_name<#input_type_params>  #where_clause
                        {
                            type Delta = #delta_type_name<#input_type_params>;

                            fn apply_delta(
                                &self,
                                delta: &Self::Delta
                            ) -> struct_delta_trait::DeltaResult<Self> {
                                Ok(Self)
                            }

                            fn delta(
                                &self,
                                rhs: &Self
                            ) -> struct_delta_trait::DeltaResult<Self::Delta> {
                                Ok(#delta_type_name)
                            }
                        }
                    }
                },
            }
        },
        DataType::Enum => {
            let mut apply_delta_tokens = TokenStream2::new();
            for (lhs_enum_variant, rhs_enum_variant)
                in iproduct!(info.enum_variants.iter(), info.enum_variants.iter())
            {
                let lhs_struct_variant = lhs_enum_variant.struct_variant;
                let rhs_struct_variant = rhs_enum_variant.struct_variant;
                let lhs_variant_name = &lhs_enum_variant.name;
                let rhs_variant_name = &rhs_enum_variant.name;
                let lhs_field_names = &lhs_enum_variant.field_names;
                let rhs_field_names = &rhs_enum_variant.field_names;
                let lhs_field_types = &lhs_enum_variant.field_types;
                let _rhs_field_types = &rhs_enum_variant.field_types;
                apply_delta_tokens.extend(match (lhs_struct_variant, rhs_struct_variant) {
                    (StructVariant::NamedStruct, StructVariant::NamedStruct)
                        if lhs_variant_name == rhs_variant_name =>
                    {
                        let lfield_names: Vec<Ident> = lhs_field_names.iter()
                            .map(|ident| format_ident!("lhs{}", ident))
                            .collect();
                        let rfield_names: Vec<Ident> = lhs_field_names.iter()
                            .map(|ident| format_ident!("delta{}", ident))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_name {
                                #(#lhs_field_names),*
                            } = self {
                                #( let #lfield_names = #lhs_field_names; )*
                                if let Self::Delta::#rhs_variant_name {
                                    #(#rhs_field_names),*
                                } = delta {
                                    #( let #rfield_names = #lhs_field_names; )*
                                    return Ok(Self::#lhs_variant_name {
                                        #(
                                            #lhs_field_names:
                                            match #rfield_names.as_ref() {
                                                None => { #lfield_names.clone() }
                                                Some(d) => {
                                                    #lfield_names.apply_delta(d)?
                                                },
                                            },
                                        )*
                                    });
                                }
                            }
                        }
                    },
                    (StructVariant::TupleStruct, StructVariant::TupleStruct)
                        if lhs_variant_name == rhs_variant_name =>
                    {
                        let field_count = lhs_field_types.len();
                        let lfield_names: Vec<Ident> = (0 .. field_count)
                            .map(|ident| format_ident!("lhs{}", ident))
                            .collect();
                        let rfield_names: Vec<Ident> = (0 .. field_count)
                            .map(|ident| format_ident!("delta{}", ident))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_name(
                                #(#lfield_names),*
                            ) = self {
                                if let Self::Delta::#rhs_variant_name(
                                    #(#rfield_names),*
                                ) = delta {
                                    return Ok(Self::#lhs_variant_name(
                                        #(
                                            match #rfield_names.as_ref() {
                                                None => #lfield_names.clone(),
                                                Some(d) =>
                                                    #lfield_names.apply_delta(d)?,
                                            },
                                        )*
                                    ));
                                }
                            }
                        }
                    },
                    (_, StructVariant::UnitStruct)
                        if lhs_variant_name == rhs_variant_name => quote! {
                            if let Self::#lhs_variant_name = self {
                                if let Self::Delta::#rhs_variant_name = delta {
                                    return Ok(Self::#lhs_variant_name);
                                }
                            }
                        },
                    _ => quote! { },
                });
            }
            for enum_variant in info.enum_variants.iter() {
                let struct_variant = enum_variant.struct_variant;
                let variant_name = &enum_variant.name;
                let field_names = &enum_variant.field_names;
                let field_types = &enum_variant.field_types;
                apply_delta_tokens.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        if let Self::Delta::#variant_name {
                            #(#field_names),*
                        } = delta {
                            use struct_delta_trait::{DeltaError, FromDelta};
                            // use std::convert::TryInto;
                            return Ok(Self::#variant_name {
                                #(
                                    #field_names: match #field_names.as_ref() {
                                        // Some(d) => d.clone().try_into()?,
                                        Some(d) => <#field_types>::from_delta(
                                            d.clone()
                                        )?,
                                        None => return Err(
                                            DeltaError::ExpectedValue
                                        )?,
                                    },
                                )*
                            })
                        }
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            if let Self::Delta::#variant_name(
                                #(#field_names),*
                            ) = delta {
                                use struct_delta_trait::{DeltaError, FromDelta};
                                return Ok(Self::#variant_name(
                                    #(
                                        match #field_names.as_ref() {
                                            Some(d) => <#field_types>::from_delta(
                                                d.clone()
                                            )?,
                                            None => return Err(
                                                DeltaError::ExpectedValue
                                            )?,
                                        },
                                    )*
                                ))
                            }
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        if let Self::Delta::#variant_name = delta {
                            return Ok(Self::#variant_name)
                        }
                    },
                });
            }

            let mut delta_tokens = TokenStream2::new();
            for (lhs_enum_variant, rhs_enum_variant)
                in iproduct!(info.enum_variants.iter(), info.enum_variants.iter())
            {
                let lhs_struct_variant = lhs_enum_variant.struct_variant;
                let rhs_struct_variant = rhs_enum_variant.struct_variant;
                let lhs_variant_name = &lhs_enum_variant.name;
                let rhs_variant_name = &rhs_enum_variant.name;
                let lhs_field_names = &lhs_enum_variant.field_names;
                let rhs_field_names = &rhs_enum_variant.field_names;
                let lhs_field_types = &lhs_enum_variant.field_types;
                let _rhs_field_types = &rhs_enum_variant.field_types;
                delta_tokens.extend(match (lhs_struct_variant, rhs_struct_variant) {
                    (StructVariant::NamedStruct, StructVariant::NamedStruct)
                        if lhs_variant_name == rhs_variant_name =>
                    {
                        let lfield_names: Vec<Ident> = lhs_field_names.iter()
                            .map(|ident| format_ident!("lhs_{}", ident))
                            .collect();
                        let rfield_names: Vec<Ident> = lhs_field_names.iter()
                            .map(|ident| format_ident!("rhs_{}", ident))
                            .collect();
                        let delta_names: Vec<Ident> = lhs_field_names.iter()
                            .map(|ident| format_ident!("{}_delta", ident))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_name {
                                #(#lhs_field_names),*
                            } = self {
                                #( let #lfield_names = #lhs_field_names; )*
                                if let Self::#rhs_variant_name {
                                    #(#rhs_field_names),*
                                } = rhs {
                                    #( let #rfield_names = #lhs_field_names; )*
                                    #(
                                        let #delta_names: Option<
                                            <#lhs_field_types as struct_delta_trait::DeltaOps>::Delta
                                        > = if #lfield_names == #rfield_names {
                                            None
                                        } else {
                                            Some(
                                                #lfield_names.delta(&#rfield_names)?
                                            )
                                        };
                                    )*
                                    return Ok(Self::Delta::#lhs_variant_name {
                                        #(
                                            #lhs_field_names: #delta_names,
                                        )*
                                    });
                                }
                            }
                        }
                    },
                    (StructVariant::TupleStruct, StructVariant::TupleStruct)
                        if lhs_variant_name == rhs_variant_name =>
                    {
                        let field_count = lhs_field_types.len();
                        let lfield_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("lhs_{}", token))
                            .collect();
                        let rfield_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("rhs_{}", token))
                            .collect();
                        let delta_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("delta_{}", token))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_name(
                                #(#lfield_names),*
                            ) = self {
                                if let Self::#rhs_variant_name(
                                    #(#rfield_names),*
                                ) = rhs {
                                    #(
                                        let #delta_names: Option<
                                            <#lhs_field_types as struct_delta_trait::DeltaOps>::Delta
                                        > = if #lfield_names != #rfield_names {
                                            Some(#lfield_names.delta(&#rfield_names)?)
                                        } else {
                                            None
                                        };
                                    )*
                                    return Ok(Self::Delta::#lhs_variant_name(
                                        #(
                                            #delta_names,
                                        )*
                                    ));
                                }
                            }
                        }
                    },
                    (_, StructVariant::UnitStruct)
                        if lhs_variant_name == rhs_variant_name => quote! {
                            if let Self::#lhs_variant_name = self {
                                if let Self::#rhs_variant_name = rhs {
                                    return Ok(Self::Delta::#rhs_variant_name);
                                }
                            }
                        },
                    _ => quote! { },
                });
            }
            for enum_variant in info.enum_variants.iter() {
                let struct_variant = enum_variant.struct_variant;
                let variant_name = &enum_variant.name;
                let field_names = &enum_variant.field_names;
                let field_types = &enum_variant.field_types;
                delta_tokens.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        if let Self::#variant_name { #(#field_names),* } = rhs {
                            use struct_delta_trait::IntoDelta;
                            return Ok(Self::Delta::#variant_name {
                                #(
                                    #field_names: Some(
                                        #field_names.clone().into_delta()?
                                    ),
                                )*
                            });
                        }
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            if let Self::#variant_name(
                                #(#field_names),*
                            ) = rhs {
                                use struct_delta_trait::IntoDelta;
                                return Ok(Self::Delta::#variant_name(
                                    #(
                                        Some(#field_names.clone().into_delta()?),
                                    )*
                                ));
                            }
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        if let Self::#variant_name = rhs {
                            return Ok(Self::Delta::#variant_name);
                        }
                    },
                });
            }

            let input_type_name = &info.input_type_name;
            let input_type_params = &info.input_type_params;
            let input_type_param_decls = &info.input_type_param_decls;
            // let input_field_names = &info.input_field_names;
            // let input_field_types = &info.input_field_types;
            let where_clause = &info.deltaops_trait_impl_where_clause;
            let delta_type_name = &info.delta_type_name;
            quote! {
                impl<#input_type_param_decls> struct_delta_trait::DeltaOps
                    for #input_type_name<#input_type_params>
                    #where_clause
                {
                    type Delta = #delta_type_name<#input_type_params>;

                    fn apply_delta(
                        &self,
                        delta: &Self::Delta
                    ) -> struct_delta_trait::DeltaResult<Self> {
                        #apply_delta_tokens
                        struct_delta_trait::bug_detected!()
                    }

                    fn delta(
                        &self,
                        rhs: &Self
                    ) -> struct_delta_trait::DeltaResult<Self::Delta> {
                        #delta_tokens
                        struct_delta_trait::bug_detected!()
                    }
                }
            }
        },
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
    }
}

#[allow(non_snake_case)]
fn impl_IntoDelta_for_input_type(info: &Info) -> TokenStream2 {
    match info.input_data_type {
        DataType::Struct => {
            let input_type_name = &info.input_type_name;
            let input_type_params = &info.input_type_params;
            let input_type_param_decls = &info.input_type_param_decls;
            let input_field_names = &info.input_field_names;
            let input_field_types = &info.input_field_types;
            let where_clause = &info.deltaops_trait_impl_where_clause;
            let delta_type_name = &info.delta_type_name;
            let mut match_body = TokenStream2::new();
            match_body.extend(match info.input_struct_variant {
                StructVariant::NamedStruct => quote! {
                    Self { #( #input_field_names ),* } => {
                        use struct_delta_trait::IntoDelta;
                        #delta_type_name {
                            #(
                                #input_field_names:
                                Some(#input_field_names.into_delta()?),
                            )*
                        }
                    },
                },
                StructVariant::TupleStruct => {
                    let field_count = input_field_types.len();
                    let field_names: Vec<Ident> = (0 .. field_count)
                        .map(|token| format_ident!("field{}", token))
                        .collect();
                    quote! {
                        Self(#( #field_names ),*) => {
                            use struct_delta_trait::IntoDelta;
                            #delta_type_name(
                                #(
                                    Some(#field_names.into_delta()?),
                                )*
                            )
                        },
                    }
                },
                StructVariant::UnitStruct => quote! {
                    Self => #delta_type_name,
                },
            });
            quote! {
                impl<#input_type_param_decls> struct_delta_trait::IntoDelta
                    for #input_type_name<#input_type_params>
                    #where_clause
                {
                    fn into_delta(self) -> struct_delta_trait::DeltaResult<
                        <Self as struct_delta_trait::DeltaOps>::Delta
                    > {
                        Ok(match self {
                            #match_body
                        })
                    }
                }
            }
        },
        DataType::Enum => {
            let input_type_name = &info.input_type_name;
            let input_type_params = &info.input_type_params;
            let input_type_param_decls = &info.input_type_param_decls;
            // let input_field_names = &info.input_field_names;
            // let input_field_types = &info.input_field_types;
            let where_clause = &info.deltaops_trait_impl_where_clause;
            let delta_type_name = &info.delta_type_name;
            let mut match_body = TokenStream2::new();
            for enum_variant in info.enum_variants.iter() {
                let struct_variant = enum_variant.struct_variant;
                let variant_name = &enum_variant.name;
                let field_names = &enum_variant.field_names;
                let field_types = &enum_variant.field_types;
                match_body.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        Self::#variant_name { #( #field_names ),* } => {
                            use struct_delta_trait::IntoDelta;
                            #delta_type_name::#variant_name {
                                #(
                                    #field_names:
                                    Some(#field_names.into_delta()?),
                                )*
                            }
                        },
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            Self::#variant_name( #( #field_names ),* ) => {
                                use struct_delta_trait::IntoDelta;
                                #delta_type_name::#variant_name(
                                    #(
                                        Some(#field_names.into_delta()?),
                                    )*
                                )
                            },
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        Self::#variant_name => {
                            #delta_type_name::#variant_name
                        },
                    },
                });
            }
            quote! {
                impl<#input_type_param_decls> struct_delta_trait::IntoDelta
                    for #input_type_name<#input_type_params>
                    #where_clause
                {
                    fn into_delta(self) -> struct_delta_trait::DeltaResult<
                        <Self as struct_delta_trait::DeltaOps>::Delta
                    > {
                        Ok(match self {
                            #match_body
                        })
                    }
                }
            }
        },
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
    }
}


#[allow(non_snake_case)]
fn impl_FromDelta_for_input_type(info: &Info) -> TokenStream2 {
    match info.input_data_type {
        DataType::Struct => {
            let input_type_name = &info.input_type_name;
            let input_type_params = &info.input_type_params;
            let input_type_param_decls = &info.input_type_param_decls;
            let input_field_names = &info.input_field_names;
            let input_field_types = &info.input_field_types;
            let where_clause = &info.deltaops_trait_impl_where_clause;
            let delta_type_name = &info.delta_type_name;
            let mut match_body = TokenStream2::new();
            match_body.extend(match info.input_struct_variant {
                StructVariant::NamedStruct => quote! {
                    #delta_type_name { #( #input_field_names ),* } => {
                        Self {
                            #(
                                #input_field_names:
                                <#input_field_types>::from_delta(
                                    #input_field_names.ok_or(
                                        DeltaError::ExpectedValue
                                    )?
                                )?,
                            )*
                        }
                    },
                },
                StructVariant::TupleStruct => {
                    let field_count = input_field_types.len();
                    let field_names: Vec<Ident> = (0 .. field_count)
                        .map(|token| format_ident!("field{}", token))
                        .collect();
                    quote! {
                        #delta_type_name( #( #field_names ),* ) => {
                            Self(
                                #(
                                    <#input_field_types>::from_delta(
                                        #field_names.ok_or(
                                            DeltaError::ExpectedValue
                                        )?
                                    )?,
                                )*
                            )
                        },
                    }
                },
                StructVariant::UnitStruct => quote! {
                    #delta_type_name => Self,
                },
            });

            quote! {
                impl<#input_type_param_decls> struct_delta_trait::FromDelta
                    for #input_type_name<#input_type_params>
                    #where_clause
                {
                    fn from_delta(
                        delta: <Self as struct_delta_trait::DeltaOps>::Delta
                    ) -> struct_delta_trait::DeltaResult<Self> {
                        use struct_delta_trait::DeltaError;
                        Ok(match delta {
                            #match_body
                        })
                    }
                }
            }
        },
        DataType::Enum => {
            let input_type_name = &info.input_type_name;
            let input_type_params = &info.input_type_params;
            let input_type_param_decls = &info.input_type_param_decls;
            // let input_field_names = &info.input_field_names;
            // let input_field_types = &info.input_field_types;
            let where_clause = &info.deltaops_trait_impl_where_clause;
            let delta_type_name = &info.delta_type_name;
            let mut match_body = TokenStream2::new();
            for enum_variant in info.enum_variants.iter() {
                let struct_variant = enum_variant.struct_variant;
                let variant_name = &enum_variant.name;
                let field_names = &enum_variant.field_names;
                let field_types = &enum_variant.field_types;
                match_body.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        #delta_type_name::#variant_name { #( #field_names ),* } => {
                            Self::#variant_name {
                                #(
                                    #field_names: <#field_types>::from_delta(
                                        #field_names.ok_or(
                                            DeltaError::ExpectedValue
                                        )?
                                    )?,
                                )*
                            }
                        },
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            #delta_type_name::#variant_name( #( #field_names ),* ) => {
                                Self::#variant_name(
                                    #(
                                        <#field_types>::from_delta(
                                            #field_names.ok_or(
                                                DeltaError::ExpectedValue
                                            )?
                                        )?,
                                    )*
                                )
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
            quote! {
                impl<#input_type_param_decls> struct_delta_trait::FromDelta
                    for #input_type_name<#input_type_params>
                    #where_clause
                {
                    fn from_delta(
                        delta: <Self as struct_delta_trait::DeltaOps>::Delta
                    ) -> struct_delta_trait::DeltaResult<Self> {
                        use struct_delta_trait::DeltaError;
                        Ok(match delta {
                            #match_body
                        })
                    }
                }
            }
        },
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
    }
}






#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataType {
    Enum,
    Struct,
    Union,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructVariant {
    /// A "named struct" i.e. a struct with named fields
    NamedStruct,
    /// A tuple struct i.e. unnamed/positional fields
    TupleStruct,
    /// A unit struct i.e. no fields at all
    UnitStruct,
}

fn define_delta_struct(
    struct_variant: StructVariant,
    delta_struct_name: &Ident,
    type_param_decls: &Punctuated<GenericParam, Comma>,
    where_clause: &WhereClause,
    field_names: &[Ident],
    field_types: &[Type],
) -> TokenStream2 {
    match struct_variant {
        StructVariant::NamedStruct => quote! {
            #[derive(Debug, PartialEq, Clone)]
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub struct #delta_struct_name<#type_param_decls> #where_clause {
                #(
                    pub(self) #field_names:
                    Option<<#field_types as struct_delta_trait::DeltaOps>::Delta>,
                )*
            }
        },
        StructVariant::TupleStruct => quote! {
            #[derive(Debug, PartialEq, Clone)]
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub struct #delta_struct_name<#type_param_decls> (
                #(
                    pub(self) Option<<#field_types as struct_delta_trait::DeltaOps>::Delta>,
                )*
            ) #where_clause ;
        },
        StructVariant::UnitStruct => quote! {
            #[derive(Debug, PartialEq, Clone)]
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub struct #delta_struct_name<#type_param_decls>
                #where_clause ;
        },
    }
}


fn define_delta_enum(
    delta_enum_name: &Ident,
    type_param_decls: &Punctuated<GenericParam, Comma>,
    where_clause: &WhereClause,
    enum_variants: &[EnumVariant]
) -> TokenStream2 {
    let enum_body: TokenStream2 = enum_variants.iter()
        .map(|enum_variant: &EnumVariant| {
            let struct_variant = enum_variant.struct_variant;
            let variant_name = &enum_variant.name;
            let field_names = &enum_variant.field_names;
            let field_types = &enum_variant.field_types;
            match struct_variant {
                StructVariant::NamedStruct =>  quote! {
                    #variant_name {
                        #(
                            #field_names:
                            Option<<#field_types as struct_delta_trait::DeltaOps>::Delta>,
                        )*
                    },
                },
                StructVariant::TupleStruct =>  quote! {
                    #variant_name(
                        #(
                            Option<<#field_types as struct_delta_trait::DeltaOps>::Delta>,
                        )*
                    ),
                },
                StructVariant::UnitStruct =>  quote! {
                    #variant_name,
                },
            }
        })
        .collect();
    quote! {
        #[derive(Debug, PartialEq, Clone)]
        #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
        pub enum #delta_enum_name<#type_param_decls> #where_clause {
            #enum_body
        }
    }
}




fn enhance_where_clause_for_deltaops_trait_impl(
    generic_params: &Punctuated<GenericParam, Comma>,
    clause: &mut WhereClause,
) -> DeriveResult<()> {
    // NOTE: Add a clause for each field `f: F` of the form
    //    `F: struct_delta_trait::Delta + serde::Serialize`
    for generic_param in generic_params.iter() {
        let field_type = generic_param_to_type(generic_param)?;
        clause.predicates.push(WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: field_type,
            colon_token: Token![:](Span2::call_site()),
            bounds: vec![ // Add type param bounds
                trait_bound(&["struct_delta_trait", "DeltaOps"]),
                trait_bound(&["struct_delta_trait", "FromDelta"]),
                trait_bound(&["struct_delta_trait", "IntoDelta"]),
                trait_bound(&["serde", "Serialize"]),
                lifetimed_trait_bound(&["serde", "Deserialize"], "de"), // TODO
                trait_bound(&["PartialEq"]),
                trait_bound(&["Clone"]),
                trait_bound(&["std", "fmt", "Debug"])
            ].into_iter().collect(),
        }));
    }
    Ok(())
}

fn enhance_where_clause_for_type_def(
    generic_params: &Punctuated<GenericParam, Comma>,
    clause: &mut WhereClause,
) -> DeriveResult<()> {
    // NOTE: Add a clause for each field `f: F` of the form
    //    `F: struct_delta_trait::Delta + serde::Serialize`
    for generic_param in generic_params.iter() {
        let field_type = generic_param_to_type(generic_param)?;
        clause.predicates.push(WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: field_type,
            colon_token: Token![:](Span2::call_site()),
            bounds: vec![ // Add type param bounds
                trait_bound(&["struct_delta_trait", "DeltaOps"]),
                trait_bound(&["PartialEq"]),
                trait_bound(&["Clone"]),
                trait_bound(&["std", "fmt", "Debug"])
            ].into_iter().collect(),
        }));
    }
    Ok(())
}


fn generic_param_to_type(generic_param: &GenericParam) -> DeriveResult<Type> {
    let type_param = match generic_param {
        GenericParam::Lifetime(_) => unimplemented!("GenericParam::Lifetime(_)"),
        GenericParam::Const(_)    => unimplemented!("GenericParam::Const(_)"),
        GenericParam::Type(type_param) => type_param,
    };
    Ok(Type::Path(
        TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: {
                    let segment = PathSegment {
                        ident: type_param.ident.clone(),
                        arguments: PathArguments::None,
                    };
                    vec![segment].into_iter().collect()
                }
            },
        }
    ))
}



fn lifetimed_trait_bound(
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


fn trait_bound(path_segments: &[&str]) -> TypeParamBound {
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



fn empty_where_clause() -> WhereClause {
    WhereClause {
        where_token: Token![where](Span2::call_site()),
        predicates: Punctuated::new(),
    }
}
