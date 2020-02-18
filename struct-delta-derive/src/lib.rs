extern crate proc_macro;

#[macro_use] mod error;

use itertools::{iproduct};
use crate::error::{DeriveResult};
use proc_macro::TokenStream;
use proc_macro2::{
    Literal as Literal2,
    // Punct as Punct2, Spacing as Spacing2,
    Span as Span2,
    TokenTree as TokenTree2, TokenStream as TokenStream2
};
use quote::{format_ident, quote};
// use std::iter::FromIterator;
use syn::{
    parse_macro_input, Token/* macro that expands to a type, not a type */,
    // Attribute,
    Data, DataStruct, DataEnum, DeriveInput,
    // Field,
    Expr,
    // Fields, FieldsNamed, FieldsUnnamed,
    GenericParam, Generics, Ident,
    Path, PathArguments, PathSegment, PredicateType,
    // TypeParam,
    Type, TypeParamBound, TraitBound, TraitBoundModifier,
    // Visibility,
    WhereClause, WherePredicate,
};
use syn::punctuated::Punctuated;
// use syn::token::Eq;
use syn::token::{
    // Add, Colon, Colon2,
    Comma
};

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
    let mut field_idents: Vec<TokenTree2> = vec![];
    let mut field_types: Vec<Type> = vec![];
    let mut struct_variant: Option<StructVariant> = None;
    let mut set_struct_variant = |variant| match struct_variant {
        None => Ok({ struct_variant = Some(variant); }),
        Some(v) if v == variant => Ok((/* NOP */)),
        _ => bug_detected!(),
    };
    #[allow(unused_assignments)] let data_type;

    type EnumVariantInfo = (StructVariant, Ident, Vec<Ident>, Vec<Type>);
    let mut enum_variants: Vec<EnumVariantInfo> = vec![];

    match input.data {
        Data::Union(_) => {
            #[allow(unused_assignments)] { data_type = DataType::Union };
            unimplemented!("Delta computation for unions is not supported.")
        },
        Data::Struct(DataStruct { fields, .. }) if fields.len() == 0 => {
            data_type = DataType::Struct;
            set_struct_variant(StructVariant::UnitStruct)?;
        },
        Data::Struct(DataStruct { fields, .. }) => {
            data_type = DataType::Struct;
            for (fidx, field) in fields.iter().enumerate() {
                if let Some(ident) = field.ident.as_ref() {
                    // A "named struct" i.e. a struct with named fields
                    field_idents.push(TokenTree2::Ident(ident.clone()));
                    field_types.push(field.ty.clone());
                    set_struct_variant(StructVariant::NamedStruct)?;
                } else { // A tuple struct i.e. unnamed/positional fields
                    field_idents.push(TokenTree2::Literal(
                        Literal2::usize_unsuffixed(fidx)
                    ));
                    field_types.push(field.ty.clone());
                    set_struct_variant(StructVariant::TupleStruct)?;
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            data_type = DataType::Enum;
            for variant in variants {
                let mut struct_variant: Option<StructVariant> =
                    if variant.fields.is_empty() {
                        Some(StructVariant::UnitStruct)
                    } else {
                        None
                    };
                let mut set_struct_variant = |variant| match struct_variant {
                    None => Ok({ struct_variant = Some(variant); }),
                    Some(v) if v == variant => Ok((/* NOP */)),
                    _ => bug_detected!(),
                };
                let variant_ident: &Ident = &variant.ident;
                let mut field_idents: Vec<Ident> = vec![];
                let mut field_types: Vec<Type> = vec![];
                for field in variant.fields.iter() {
                    field_types.push(field.ty.clone());
                    if let Some(field_ident) = field.ident.as_ref() {
                        field_idents.push(field_ident.clone());
                        set_struct_variant(StructVariant::NamedStruct)?;
                    } else {
                        set_struct_variant(StructVariant::TupleStruct)?;
                    }
                }
                enum_variants.push((
                    struct_variant.unwrap(/* TODO */),
                    variant_ident.clone(),
                    field_idents,
                    field_types
                ));

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
    }

    let input_type_name: Ident = input.ident;
    let mut generics: Generics = input.generics;
    let where_clause: &mut Option<WhereClause> = &mut generics.where_clause;
    add_type_paramn_bounds_to_where_clause(where_clause, &field_types);
    let where_clause: Option<&WhereClause> = generics.where_clause.as_ref();
    let input_type_param_decls: &Punctuated<GenericParam, Comma> =
        &generics.params;
    let input_type_params: Punctuated<TokenTree2, Comma> = generics.type_params()
        .map(|type_param| type_param.ident.clone())
        .map(TokenTree2::from)
        .collect();
    let delta_type_name: Ident = format_ident!("{}Delta", input_type_name);
    let mut enum_where_clause: Option<WhereClause> = generics.where_clause.clone();
    let delta_type_definition = match data_type {
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
        DataType::Struct => define_delta_struct(
            struct_variant.unwrap(/*TODO Option*/),
            &delta_type_name,
            input_type_param_decls,
            where_clause,
            &field_idents,
            &field_types
        ),
        DataType::Enum => {
            define_delta_enum(
                &delta_type_name,
                input_type_param_decls,
                &mut enum_where_clause,
                &enum_variants
            )
        },
    };

    #[allow(non_snake_case)]
    let impl_DeltaOps_for_input_type = match data_type {
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
        DataType::Struct => quote! {
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
                            #field_idents:
                            if let Some(field_delta) = &delta.#field_idents {
                                self.#field_idents.apply_delta(&field_delta)?
                            } else {
                                self.#field_idents.clone()
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
                            #field_idents:
                            if self.#field_idents != rhs.#field_idents {
                                Some(self.#field_idents.delta(&rhs.#field_idents)?)
                            } else {
                                None
                            }
                        ),*
                    })
                }
            }
        },
        DataType::Enum => {
            let mut apply_delta_tokens = TokenStream2::new();
            for ((lhs_struct_variant,   lhs_variant_ident,
                  lhs_field_idents,     lhs_field_types),
                 (rhs_struct_variant,   rhs_variant_ident,
                  rhs_field_idents,    _rhs_field_types))
                in iproduct!(enum_variants.iter(), enum_variants.iter())
            {
                apply_delta_tokens.extend(match (lhs_struct_variant, rhs_struct_variant) {
                    (StructVariant::NamedStruct, StructVariant::NamedStruct)
                        if lhs_variant_ident == rhs_variant_ident =>
                    {
                        let lfield_idents: Vec<Ident> = lhs_field_idents.iter()
                            .map(|ident| format_ident!("lhs{}", ident))
                            .collect();
                        let rfield_idents: Vec<Ident> = lhs_field_idents.iter()
                            .map(|ident| format_ident!("delta{}", ident))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_ident {
                                #(#lhs_field_idents),*
                            } = self {
                                #( let #lfield_idents = #lhs_field_idents; )*
                                if let Self::Delta::#rhs_variant_ident {
                                    #(#rhs_field_idents),*
                                } = delta {
                                    #( let #rfield_idents = #lhs_field_idents; )*
                                    return Ok(Self::#lhs_variant_ident {
                                        #(
                                            #lhs_field_idents:
                                            match #rfield_idents.as_ref() {
                                                None => { #lfield_idents.clone() }
                                                Some(d) => {
                                                    #lfield_idents.apply_delta(d)?
                                                },
                                            },
                                        )*
                                    });
                                }
                            }
                        }
                    },
                    (StructVariant::TupleStruct, StructVariant::TupleStruct)
                        if lhs_variant_ident == rhs_variant_ident =>
                    {
                        let field_count = lhs_field_types.len();
                        let lfield_idents: Vec<Ident> = (0 .. field_count)
                            .map(|ident| format_ident!("lhs{}", ident))
                            .collect();
                        let rfield_idents: Vec<Ident> = (0 .. field_count)
                            .map(|ident| format_ident!("delta{}", ident))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_ident(
                                #(#lfield_idents),*
                            ) = self {
                                if let Self::Delta::#rhs_variant_ident(
                                    #(#rfield_idents),*
                                ) = delta {
                                    return Ok(Self::#lhs_variant_ident(
                                        #(
                                            match #rfield_idents.as_ref() {
                                                None => #lfield_idents.clone(),
                                                Some(d) =>
                                                    #lfield_idents.apply_delta(d)?,
                                            },
                                        )*
                                    ));
                                }
                            }
                        }
                    },
                    (_, StructVariant::UnitStruct)
                        if lhs_variant_ident == rhs_variant_ident => quote! {
                            if let Self::#lhs_variant_ident = self {
                                if let Self::Delta::#rhs_variant_ident = delta {
                                    return Ok(Self::#lhs_variant_ident);
                                }
                            }
                        },
                    _ => quote! { },
                });
            }
            for (struct_variant, variant_ident, field_idents, field_types)
                in enum_variants.iter()
            {
                apply_delta_tokens.extend(match struct_variant {
                    StructVariant::NamedStruct => {
                        quote! {
                            if let Self::Delta::#variant_ident {
                                #(#field_idents),*
                            } = delta {
                                use struct_delta_trait::DeltaError;
                                use std::convert::TryInto;
                                return Ok(Self::#variant_ident {
                                    #(
                                        #field_idents:
                                        match #field_idents.as_ref() {
                                            Some(d) => d.clone().try_into()?,
                                            None => return Err(DeltaError::ExpectedValue)?,
                                        },
                                    )*
                                })
                            }
                        }
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            if let Self::Delta::#variant_ident(
                                #(#field_idents),*
                            ) = delta {
                                use struct_delta_trait::DeltaError;
                                use std::convert::TryInto;
                                return Ok(Self::#variant_ident(
                                    #(
                                        match #field_idents.as_ref() {
                                            Some(d) => d.clone().try_into()?,
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
                        if let Self::Delta::#variant_ident = delta {
                            return Ok(Self::#variant_ident)
                        }
                    },
                });
            }

            let mut delta_tokens = TokenStream2::new();
            for ((lhs_struct_variant,   lhs_variant_ident,
                  lhs_field_idents,     lhs_field_types),
                 (rhs_struct_variant,   rhs_variant_ident,
                  rhs_field_idents,    _rhs_field_types))
                in iproduct!(enum_variants.iter(), enum_variants.iter())
            {
                delta_tokens.extend(match (lhs_struct_variant, rhs_struct_variant) {
                    (StructVariant::NamedStruct, StructVariant::NamedStruct)
                        if lhs_variant_ident == rhs_variant_ident =>
                    {
                        let lfield_idents: Vec<Ident> = lhs_field_idents.iter()
                            .map(|ident| format_ident!("lhs_{}", ident))
                            .collect();
                        let rfield_idents: Vec<Ident> = lhs_field_idents.iter()
                            .map(|ident| format_ident!("rhs_{}", ident))
                            .collect();
                        let delta_idents: Vec<Ident> = lhs_field_idents.iter()
                            .map(|ident| format_ident!("{}_delta", ident))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_ident {
                                #(#lhs_field_idents),*
                            } = self {
                                #( let #lfield_idents = #lhs_field_idents; )*
                                if let Self::#rhs_variant_ident {
                                    #(#rhs_field_idents),*
                                } = rhs {
                                    #( let #rfield_idents = #lhs_field_idents; )*
                                    #(
                                        let #delta_idents: Option<
                                            <#lhs_field_types as DeltaOps>::Delta
                                        > = if #lfield_idents == #rfield_idents {
                                            None
                                        } else {
                                            Some(
                                                #lfield_idents.delta(&#rfield_idents)?
                                            )
                                        };
                                    )*
                                    return Ok(Self::Delta::#lhs_variant_ident {
                                        #(
                                            #lhs_field_idents: #delta_idents,
                                        )*
                                    });
                                }
                            }
                        }
                    },
                    (StructVariant::TupleStruct, StructVariant::TupleStruct)
                        if lhs_variant_ident == rhs_variant_ident =>
                    {
                        let field_count = lhs_field_types.len();
                        let lfield_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("lhs_{}", token))
                            .collect();
                        let rfield_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("rhs_{}", token))
                            .collect();
                        let delta_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("delta_{}", token))
                            .collect();
                        quote! {
                            if let Self::#lhs_variant_ident(
                                #(#lfield_idents),*
                            ) = self {
                                if let Self::#rhs_variant_ident(
                                    #(#rfield_idents),*
                                ) = rhs {
                                    #(
                                        let #delta_idents: Option<
                                            <#lhs_field_types as DeltaOps>::Delta
                                        > = if #lfield_idents == #rfield_idents {
                                            None
                                        } else {
                                            Some(#lfield_idents.delta(&#rfield_idents)?)
                                        };
                                    )*
                                    return Ok(Self::Delta::#lhs_variant_ident(
                                        #(
                                            #delta_idents,
                                        )*
                                    ));
                                }
                            }
                        }
                    },
                    (_, StructVariant::UnitStruct)
                        if lhs_variant_ident == rhs_variant_ident => quote! {
                            if let Self::#lhs_variant_ident = self {
                                if let Self::#rhs_variant_ident = rhs {
                                    return Ok(Self::Delta::#rhs_variant_ident);
                                }
                            }
                        },
                    _ => quote! { },
                });
            }
            for (struct_variant, variant_ident, field_idents, field_types)
                in enum_variants.iter()
            {
                delta_tokens.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        if let Self::#variant_ident { #(#field_idents),* } = rhs {
                            use std::convert::TryInto;
                            return Ok(Self::Delta::#variant_ident {
                                #(
                                    #field_idents: Some(
                                        #field_idents.clone().try_into()?
                                    ),
                                )*
                            });
                        }
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            if let Self::#variant_ident(
                                #(#field_idents),*
                            ) = rhs {
                                use std::convert::TryInto;
                                return Ok(Self::Delta::#variant_ident(
                                    #(
                                        Some(#field_idents.clone().try_into()?),
                                    )*
                                ));
                            }
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        if let Self::#variant_ident = rhs {
                            return Ok(Self::Delta::#variant_ident);
                        }
                    },

                });
            }

            quote! {
                impl<#input_type_param_decls> struct_delta_trait::DeltaOps
                    for #input_type_name<#input_type_params>
                    #enum_where_clause
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
    };
    #[allow(non_snake_case)]
    let impl_TryFrom_input_type_for_delta_type = match data_type {
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
        DataType::Struct => {
            let mut match_body = TokenStream2::new();
            match_body.extend(match struct_variant.unwrap(/* TODO */) {
                StructVariant::NamedStruct => quote! {
                    #input_type_name {
                        #( #field_idents ),*
                    } => {
                        use std::convert::TryInto;
                        Self {
                            #(
                                #field_idents: Some(#field_idents.try_into()?),
                            )*
                        }
                    },
                },
                StructVariant::TupleStruct => {
                    let field_count = field_types.len();
                    let field_idents: Vec<Ident> = (0 .. field_count)
                        .map(|token| format_ident!("field{}", token))
                        .collect();
                    quote! {
                        #input_type_name(
                            #( #field_idents ),*
                        ) => {
                            use std::convert::TryInto;
                            Self(
                                #(
                                    Some(#field_idents.try_into()?),
                                )*
                            )
                        },
                    }
                },
                StructVariant::UnitStruct => quote! {
                    #input_type_name => Self,
                },
            });
            quote! {
                impl<#input_type_param_decls>
                    std::convert::TryFrom<#input_type_name<#input_type_params>>
                    for #delta_type_name<#input_type_params>
                    #where_clause
                {
                    type Error = struct_delta_trait::DeltaError;
                    fn try_from(
                        thing: #input_type_name<#input_type_params>
                    ) -> Result<Self, Self::Error> {
                        Ok(match thing {
                            #match_body
                        })
                    }
                }
            }
        },
        DataType::Enum => {
            let mut match_body = TokenStream2::new();
            for (struct_variant, variant_ident, field_idents, field_types)
                in enum_variants.iter()
            {
                match_body.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        #input_type_name::#variant_ident {
                            #( #field_idents ),*
                        } => {
                            use std::convert::TryInto;
                            Self::#variant_ident {
                                #(
                                    #field_idents: Some(#field_idents.try_into()?),
                                )*
                            }
                        },
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            #input_type_name::#variant_ident(
                                #( #field_idents ),*
                            ) => {
                                use std::convert::TryInto;
                                Self::#variant_ident(
                                    #(
                                        Some(#field_idents.try_into()?),
                                    )*
                                )
                            },
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        #input_type_name::#variant_ident => {
                            Self::#variant_ident
                        },
                    },
                });
            }
            quote! {
                impl<#input_type_param_decls>
                    std::convert::TryFrom<#input_type_name<#input_type_params>>
                    for #delta_type_name<#input_type_params>
                    #enum_where_clause
                {
                    type Error = struct_delta_trait::DeltaError;
                    fn try_from(
                        thing: #input_type_name<#input_type_params>
                    ) -> Result<Self, Self::Error> {
                        Ok(match thing {
                            #match_body
                        })
                    }
                }
            }
        },
    };
    #[allow(non_snake_case)]
    let impl_TryFrom_delta_type_for_input_type = match data_type {
        DataType::Union =>
            unimplemented!("Delta computation for unions is not supported."),
        DataType::Struct => {
            let mut match_body = TokenStream2::new();
            match_body.extend(match struct_variant.unwrap(/* TODO */) {
                StructVariant::NamedStruct => quote! {
                    #delta_type_name { #( #field_idents ),* } => {
                        use std::convert::TryInto;
                        use struct_delta_trait::DeltaError;
                        Self {
                            #(
                                #field_idents: #field_idents
                                    .ok_or(DeltaError::ExpectedValue)
                                    .map(|val| val.try_into())??,
                            )*
                        }
                    },
                },
                StructVariant::TupleStruct => {
                    let field_count = field_types.len();
                    let field_idents: Vec<Ident> = (0 .. field_count)
                        .map(|token| format_ident!("field{}", token))
                        .collect();
                    quote! {
                        #delta_type_name( #( #field_idents ),* ) => {
                            use std::convert::TryInto;
                            use struct_delta_trait::DeltaError;
                            Self(
                                #(
                                    #field_idents
                                        .ok_or(DeltaError::ExpectedValue)
                                        .map(|val| val.try_into())??,
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
                impl<#input_type_param_decls>
                    std::convert::TryFrom<#delta_type_name<#input_type_params>>
                    for #input_type_name<#input_type_params>
                    #where_clause
                {
                    type Error = struct_delta_trait::DeltaError;
                    fn try_from(
                        delta: #delta_type_name<#input_type_params>
                    ) -> Result<Self, Self::Error> {
                        Ok(match delta {
                            #match_body
                        })
                    }
                }
            }
        },
        DataType::Enum => {
            let mut match_body = TokenStream2::new();
            for (struct_variant, variant_ident, field_idents, field_types)
                in enum_variants.iter()
            {
                match_body.extend(match struct_variant {
                    StructVariant::NamedStruct => quote! {
                        #delta_type_name::#variant_ident { #( #field_idents ),* } => {
                            use std::convert::{TryFrom, TryInto};
                            use struct_delta_trait::DeltaError;
                            Self::#variant_ident {
                                #(
                                    #field_idents: #field_idents
                                        .ok_or(DeltaError::ExpectedValue)
                                        .map(|val| val.try_into())??,
                                )*
                            }
                        },
                    },
                    StructVariant::TupleStruct => {
                        let field_count = field_types.len();
                        let field_idents: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field{}", token))
                            .collect();
                        quote! {
                            #delta_type_name::#variant_ident( #( #field_idents ),* ) => {
                                use std::convert::{TryFrom, TryInto};
                                use struct_delta_trait::DeltaError;
                                Self::#variant_ident(
                                    #(
                                        #field_idents
                                            .ok_or(DeltaError::ExpectedValue)
                                            .map(|val| val.try_into())??,
                                    )*
                                )
                            },
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        #delta_type_name::#variant_ident => {
                            Self::#variant_ident
                        },
                    },
                });
            }
            quote! {
                impl<#input_type_param_decls>
                    std::convert::TryFrom<#delta_type_name<#input_type_params>>
                    for #input_type_name<#input_type_params>
                    #enum_where_clause
                {
                    type Error = struct_delta_trait::DeltaError;
                    fn try_from(
                        delta: #delta_type_name<#input_type_params>
                    ) -> Result<Self, Self::Error> {
                        Ok(match delta {
                            #match_body
                        })
                    }
                }
            }
        },
    };

    let output: TokenStream2 = quote! {
        #delta_type_definition

        #impl_DeltaOps_for_input_type

        #impl_TryFrom_input_type_for_delta_type
        #impl_TryFrom_delta_type_for_input_type
    };

    println!("{}", output);
    Ok(output)
}






#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataType {
    Enum,
    Struct,
    Union,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructVariant {
    UnitStruct,
    TupleStruct,
    NamedStruct
}

fn define_delta_struct(
    struct_variant: StructVariant,
    delta_struct_name: &Ident,
    type_param_decls: &Punctuated<GenericParam, Comma>,
    where_clause: Option<&WhereClause>,
    field_idents: &[TokenTree2],
    field_types: &[Type],
) -> TokenStream2 {
    match struct_variant {
        StructVariant::NamedStruct => quote! {
            #[derive(Debug, PartialEq, Clone)]
            pub struct #delta_struct_name<#type_param_decls> #where_clause {
                #(
                    pub(self) #field_idents: Option<<#field_types as DeltaOps>::Delta>,
                )*
            }
        },
        StructVariant::TupleStruct => quote! {
            #[derive(Debug, PartialEq, Clone)]
            pub struct #delta_struct_name<#type_param_decls> (
                #(
                    pub(self) Option<<#field_types as DeltaOps>::Delta>,
                )*
            ) #where_clause ;
        },
        StructVariant::UnitStruct => quote! {
            #[derive(Debug, PartialEq, Clone)]
            pub struct #delta_struct_name<#type_param_decls>
                #where_clause ;
        },
    }
}


fn define_delta_enum(
    delta_enum_name: &Ident,
    type_param_decls: &Punctuated<GenericParam, Comma>,
    where_clause: &mut Option<WhereClause>,
    enum_variants: &[(StructVariant, Ident, Vec<Ident>, Vec<Type>)]
) -> TokenStream2 {
    let mut enum_body = TokenStream2::new();
    for (variant, name, field_idents, field_types) in enum_variants {
        add_type_paramn_bounds_to_where_clause(where_clause, field_types);
        enum_body.extend(generate_enum_variant(
            *variant,
            name,
            &field_idents,
            &field_types
        ));
    }
    quote! {
        #[derive(Debug, PartialEq, Clone)]
        pub enum #delta_enum_name<#type_param_decls> #where_clause {
            #enum_body
        }
    }
}

fn generate_enum_variant(
    struct_variant: StructVariant,
    variant_name: &Ident,
    field_idents: &[Ident],
    field_types: &[Type],
) -> TokenStream2 {
    match struct_variant {
        StructVariant::NamedStruct =>  quote! {
            #variant_name {
                #( #field_idents: Option<<#field_types as DeltaOps>::Delta>, )*
            },
        },
        StructVariant::TupleStruct =>  quote! {
            #variant_name(
                #( Option<<#field_types as DeltaOps>::Delta>, )*
            ),
        },
        StructVariant::UnitStruct =>  quote! {
            #variant_name,
        },
    }
}





fn add_type_paramn_bounds_to_where_clause(
    where_clause: &mut Option<WhereClause>,
    field_types: &[Type],
) {
    if where_clause.is_none() {
        // NOTE: initialize the `WhereClause` if there isn't one yet
        *where_clause = Some(WhereClause {
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
    if let Some(ref mut clause) = where_clause {
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
                    trait_bound(&["PartialEq"]),
                    trait_bound(&["Clone"]),
                    trait_bound(&["std", "fmt", "Debug"])
                ].into_iter().collect(),
            }));
        }
    }
}
