//!

use crate::{DeriveError, DeriveResult};
use itertools::iproduct;
use proc_macro2::{
    Ident as Ident2, Literal as Literal2, Span as Span2,
    TokenStream as TokenStream2
};
use syn::*;
use syn::punctuated::*;
use syn::token::Comma;
use quote::{format_ident, quote};


#[derive(Clone, Debug)]
pub enum UserDefinedTypeDesc {
    #[allow(unused)]
    Enum {
        /// The input enum's type name
        type_name: Ident2,
        /// The name of the generated delta type
        delta_name: Ident2,
        /// A description of the input enum's variants
        variants: EnumVariantsDesc,
        /// The input enum's type parameter declarations e.g. <T: Copy, U, V>
        input_type_param_decls: Punctuated<GenericParam, Comma>,
        /// The input enum's type parameters e.g. <T, U, V>
        input_type_params: Punctuated<Ident, Comma>,
        /// WhereClause for generated type definitions
        type_def_where_clause: WhereClause,
        /// WhereClause for the generated `DeltaOps` impl
        deltaops_trait_impl_where_clause: WhereClause,
    },
    #[allow(unused)]
    Struct {
        /// Indicates whether it is a named struct, tuple struct or unit struct
        struct_variant: StructVariant,
        /// The struct's type name
        type_name: Ident2,
        /// The name of the generated delta type
        delta_name: Ident2,
        /// A description of the struct's fields
        fields: FieldsDesc,
        /// The input enum's type parameter declarations e.g. <T: Copy, U, V>
        input_type_param_decls: Punctuated<GenericParam, Comma>,
        /// The input enum's type parameters e.g. <T, U, V>
        input_type_params: Punctuated<Ident, Comma>,
        /// WhereClause for generated type definitions
        type_def_where_clause: WhereClause,
        /// WhereClause for the generated `DeltaOps` impl
        deltaops_trait_impl_where_clause: WhereClause,
    },
    #[doc(hidden)]
    Union,
}

impl UserDefinedTypeDesc {
    pub fn parse(input: &DeriveInput) -> DeriveResult<Self> {
        let mut new: Self = Self::Union; // placeholder initial value
        match &input.data {
            Data::Struct(DataStruct { fields, .. }) if !fields.is_empty() =>
                new.parse_struct(input, fields)?,
            Data::Struct(DataStruct { .. }) =>
                new.parse_unit_struct(input)?,
            Data::Enum(DataEnum { variants, .. }) =>
                new.parse_enum(input, variants)?,
            Data::Union(_) => {/* NOP */},
        }
        Ok(new)
    }

    fn parse_struct(
        &mut self,
        input: &DeriveInput,
        input_fields: &Fields,
    ) -> DeriveResult<()> {
        if !self.is_struct() { *self = Self::new_struct(input); }
        if let Self::Struct {
            struct_variant,
            fields,
            input_type_param_decls,
            ref mut type_def_where_clause,
            ref mut deltaops_trait_impl_where_clause,
            ..
        } = self {
            for (fidx, field) in input_fields.iter().enumerate() {
                match field.ident.as_ref() {
                    Some(field_ident) => {
                        *struct_variant = StructVariant::NamedStruct;
                        fields.add(FieldDesc::Named {
                            name: field_ident.clone(),
                            ty: field.ty.clone(),
                            ignore_field: ignore_field(field),
                        });
                    },
                    None => {
                        *struct_variant = StructVariant::TupleStruct;
                        fields.add(FieldDesc::Positional {
                            position: Literal2::usize_unsuffixed(fidx),
                            ty: field.ty.clone(),
                            ignore_field: ignore_field(field),
                        });
                    },
                }
            }
            Self::enhance_where_clause_for_type_definition(
                &input_type_param_decls,
                type_def_where_clause
            )?;
            Self::enhance_where_clause_for_deltaops_trait_impl(
                &input_type_param_decls,
                deltaops_trait_impl_where_clause,
            )?;
        }
        Ok(())
    }

    fn parse_unit_struct(&mut self, input: &DeriveInput) -> DeriveResult<()> {
        if !self.is_struct() { *self = Self::new_struct(input); }
        if let Self::Struct {
            struct_variant,
            input_type_param_decls,
            ref mut type_def_where_clause,
            ref mut deltaops_trait_impl_where_clause,
            ..
        } = self {
            *struct_variant = StructVariant::UnitStruct;
            Self::enhance_where_clause_for_type_definition(
                &input_type_param_decls,
                type_def_where_clause,
            )?;
            Self::enhance_where_clause_for_deltaops_trait_impl(
                &input_type_param_decls,
                deltaops_trait_impl_where_clause,
            )?;
        }
        Ok(())
    }

    fn parse_enum(
        &mut self,
        input: &DeriveInput,
        input_enum_variants: &Punctuated<Variant, Comma>,
    ) -> DeriveResult<()> {
        if !self.is_enum() { *self = Self::new_enum(input); }
        if let Self::Enum {
            variants,
            input_type_param_decls,
            ref mut type_def_where_clause,
            ref mut deltaops_trait_impl_where_clause,
            ..
        } = self {
            for iev in input_enum_variants {
                let mut variant = EnumVariantDesc::new(&iev.ident);
                for (fidx, field) in iev.fields.iter().enumerate() {
                    match field.ident.as_ref() {
                        Some(field_ident) => {
                            variant.struct_variant = StructVariant::NamedStruct;
                            variant.add_field(FieldDesc::Named {
                                name: field_ident.clone(),
                                ty: field.ty.clone(),
                                ignore_field: ignore_field(field),
                            });
                        },
                        None => {
                            variant.struct_variant = StructVariant::TupleStruct;
                            variant.add_field(FieldDesc::Positional {
                                position: Literal2::usize_unsuffixed(fidx),
                                ty: field.ty.clone(),
                                ignore_field: ignore_field(field),
                            });
                        },
                    }
                }
                variants.add_variant(variant);
            }
            Self::enhance_where_clause_for_type_definition(
                &input_type_param_decls,
                type_def_where_clause,
            )?;
            Self::enhance_where_clause_for_deltaops_trait_impl(
                &input_type_param_decls,
                deltaops_trait_impl_where_clause,
            )?;
        }
        Ok(())
    }

    fn enhance_where_clause_for_type_definition(
        generic_params: &Punctuated<GenericParam, Comma>,
        clause: &mut WhereClause,
    ) -> DeriveResult<()> {
        // NOTE: Add a clause for each field `f: F` of the form
        //    `F: deltoid::Delta (+ <Trait>)*`
        for generic_param in generic_params.iter() {
            clause.predicates.push(WherePredicate::Type(PredicateType {
                lifetimes: None,
                bounded_ty: Self::generic_param_to_type(generic_param)?,
                colon_token: Token![:](Span2::call_site()),
                bounds: vec![ // Add type param bounds
                    Self::trait_bound(&["deltoid", "DeltaOps"]),
                    Self::trait_bound(&["PartialEq"]),
                    Self::trait_bound(&["Clone"]),
                    Self::trait_bound(&["std", "fmt", "Debug"])
                ].into_iter().collect(),
            }));
        }
        Ok(())
    }

    fn enhance_where_clause_for_deltaops_trait_impl(
        generic_params: &Punctuated<GenericParam, Comma>,
        clause: &mut WhereClause,
    ) -> DeriveResult<()> {
        // NOTE: Add a clause for each field `f: F` of the form
        //    `F: deltoid::Delta + serde::Serialize`
        for generic_param in generic_params.iter() {
            let field_type = Self::generic_param_to_type(generic_param)?;
            clause.predicates.push(WherePredicate::Type(PredicateType {
                lifetimes: None,
                bounded_ty: field_type,
                colon_token: Token![:](Span2::call_site()),
                bounds: vec![ // Add type param bounds
                    Self::trait_bound(&["deltoid", "DeltaOps"]),
                    Self::trait_bound(&["deltoid", "FromDelta"]),
                    Self::trait_bound(&["deltoid", "IntoDelta"]),
                    Self::trait_bound(&["serde", "Serialize"]),
                    Self::lifetimed_trait_bound(&["serde", "Deserialize"], "de"),
                    Self::trait_bound(&["PartialEq"]),
                    Self::trait_bound(&["Clone"]),
                    Self::trait_bound(&["std", "fmt", "Debug"])
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



    fn new_enum(input: &DeriveInput) -> Self {
        Self::Enum {
            type_name: input.ident.clone(),
            delta_name: format_ident!("{}Delta", &input.ident),
            variants: EnumVariantsDesc::new(),
            input_type_param_decls: input.generics.params.clone(),
            input_type_params: input.generics.type_params()
                .map(|type_param| type_param.ident.clone())
                .collect(),
            type_def_where_clause: input.generics.where_clause.clone()
                .unwrap_or_else(empty_where_clause),
            deltaops_trait_impl_where_clause: input.generics.where_clause.clone()
                .unwrap_or_else(empty_where_clause),
        }
    }

    fn new_struct(input: &DeriveInput) -> Self {
        Self::Struct {
            struct_variant: StructVariant::UnitStruct,
            type_name: input.ident.clone(),
            delta_name: format_ident!("{}Delta", &input.ident),
            fields: FieldsDesc::new(),
            input_type_param_decls: input.generics.params.clone(),
            input_type_params: input.generics.type_params()
                .map(|type_param| type_param.ident.clone())
                .collect(),
            type_def_where_clause: input.generics.where_clause.clone()
                .unwrap_or_else(empty_where_clause),
            deltaops_trait_impl_where_clause: input.generics.where_clause.clone()
                .unwrap_or_else(empty_where_clause),
        }
    }

    pub fn is_enum(&self) -> bool {
        match self {
            Self::Enum { .. } => true,
            _ => false
        }
    }

    pub fn is_struct(&self) -> bool {
        match self {
            Self::Struct { .. } => true,
            _ => false
        }
    }

    pub fn define_delta_type(&self) -> DeriveResult<TokenStream2> {
        Ok(match self {
            Self::Struct {
                struct_variant,
                delta_name,
                fields,
                input_type_param_decls,
                type_def_where_clause,
                ..
            } => Self::define_delta_struct(
                *struct_variant,
                delta_name,
                &fields,
                input_type_param_decls,
                type_def_where_clause,
            )?,
            Self::Enum {
                delta_name,
                variants,
                input_type_param_decls,
                type_def_where_clause,
                ..
            } => Self::define_delta_enum(
                delta_name,
                variants,
                input_type_param_decls,
                type_def_where_clause,
            )?,
            Self::Union => quote! {/*NOP*/},
        })
    }

    #[allow(non_snake_case)]
    pub fn define_DeltaOps_impl(&self) -> DeriveResult<TokenStream2> {
        let accumulate_tokens = |total_result, elt_result| {
            let (total, elt) = (total_result?, elt_result?);
            Ok(quote! { #total  #elt })
        };
        Ok(match self {
            Self::Struct {
                struct_variant,
                type_name,
                delta_name: delta_type_name,
                fields,
                input_type_param_decls,
                input_type_params,
                deltaops_trait_impl_where_clause: where_clause,
                ..
            } => match struct_variant {
                StructVariant::NamedStruct => {
                    let apply_delta_field_assignments: TokenStream2 = fields.iter()
                        .map(|field: &FieldDesc| {
                            let fname = field.name_ref()?;
                            if field.ignore_field() {
                                return Ok(quote! { #fname: self.#fname.clone(), });
                            }
                            Ok(quote! {
                                #fname: if let Some(d) = &delta.#fname {
                                    self.#fname.apply_delta(&d)?
                                } else {
                                    self.#fname.clone()
                                },
                            })
                        })
                        .fold(Ok(quote!{ }), accumulate_tokens)?;
                    let delta_field_assignments: TokenStream2 = fields.iter()
                        .map(|field: &FieldDesc| {
                            let fname = field.name_ref()?;
                            if field.ignore_field() {
                                return Ok(quote! {
                                    #fname: std::marker::PhantomData,
                                });
                            }
                            Ok(quote! {
                                #fname: if self.#fname != rhs.#fname {
                                    Some(self.#fname.delta(&rhs.#fname)?)
                                } else {
                                    None
                                },
                            })
                        })
                        .fold(Ok(quote!{ }), accumulate_tokens)?;
                    quote! {
                        impl<#input_type_param_decls>
                            deltoid::DeltaOps
                            for #type_name<#input_type_params>
                            #where_clause
                        {
                            type Delta = #delta_type_name<#input_type_params>;

                            #[allow(unused)]
                            fn apply_delta(&self, delta: &Self::Delta) ->
                                deltoid::DeltaResult<Self>
                            {
                                Ok(Self { #apply_delta_field_assignments })
                            }

                            #[allow(unused)]
                            fn delta(&self, rhs: &Self) ->
                                deltoid::DeltaResult<Self::Delta>
                            {
                                use deltoid::IntoDelta;
                                Ok(#delta_type_name { #delta_field_assignments })
                            }
                        }
                    }
                },
                StructVariant::TupleStruct => {
                    let apply_delta_field_assignments: TokenStream2 = fields.iter()
                        .map(|field: &FieldDesc| {
                            let fpos = field.pos_ref()?;
                            if field.ignore_field() {
                                return Ok(quote! { self.#fpos.clone(), });
                            }
                            Ok(quote! {
                                if let Some(d) = &delta.#fpos {
                                    self.#fpos.apply_delta(&d)?
                                } else {
                                    self.#fpos.clone()
                                },
                            })
                        })
                        .fold(Ok(quote!{ }), accumulate_tokens)?;
                    let delta_field_assignments: TokenStream2 = fields.iter()
                        .map(|field: &FieldDesc| {
                            let fpos = field.pos_ref()?;
                            if field.ignore_field() {
                                return Ok(quote! { std::marker::PhantomData, });
                            }
                            Ok(quote! {
                                if self.#fpos != rhs.#fpos {
                                    Some(self.#fpos.delta(&rhs.#fpos)?)
                                } else {
                                    None
                                },
                            })
                        })
                        .fold(Ok(quote!{ }), accumulate_tokens)?;
                    quote! {
                        impl<#input_type_param_decls>
                            deltoid::DeltaOps
                            for #type_name<#input_type_params>
                            #where_clause
                        {
                            type Delta = #delta_type_name<#input_type_params>;

                            #[allow(unused)]
                            fn apply_delta(&self, delta: &Self::Delta) ->
                                deltoid::DeltaResult<Self>
                            {
                                Ok(Self( #apply_delta_field_assignments ))
                            }

                            #[allow(unused)]
                            fn delta(&self,rhs: &Self) ->
                                deltoid::DeltaResult<Self::Delta>
                            {
                                use deltoid::IntoDelta;
                                Ok(#delta_type_name(#delta_field_assignments))
                            }
                        }
                    }
                },
                StructVariant::UnitStruct => quote! {
                    impl<#input_type_param_decls>
                        deltoid::DeltaOps
                        for #type_name<#input_type_params>
                        #where_clause
                    {
                        type Delta = #delta_type_name<#input_type_params>;

                        #[allow(unused)]
                        fn apply_delta(&self, delta: &Self::Delta) ->
                            deltoid::DeltaResult<Self>
                        {
                            Ok(Self)
                        }

                        #[allow(unused)]
                        fn delta(&self,rhs: &Self) ->
                            deltoid::DeltaResult<Self::Delta>
                        {
                            Ok(#delta_type_name)
                        }
                    }
                },
            },
            Self::Enum {
                type_name,
                delta_name: delta_type_name,
                variants,
                input_type_param_decls,
                input_type_params,
                deltaops_trait_impl_where_clause: where_clause,
                ..
            } => {
                let mut apply_delta_tokens = TokenStream2::new();
                for (lhs_variant, rhs_variant) in
                    iproduct!(variants.iter(),  variants.iter())
                {
                    let lhs_variant_name = &lhs_variant.name;
                    let rhs_variant_name = &rhs_variant.name;
                    let field_types: Vec<&Type> = lhs_variant.fields()
                        .map(|field: &FieldDesc| field.type_ref())
                        .collect();
                    apply_delta_tokens.extend(match (
                            lhs_variant.struct_variant,
                            rhs_variant.struct_variant
                    ) {
                        (StructVariant::NamedStruct, StructVariant::NamedStruct)
                            if lhs_variant_name == rhs_variant_name =>
                        {
                            let field_names: Vec<&Ident2> = lhs_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref().unwrap())
                                .collect();
                            let lhs_names: Vec<Ident2> = lhs_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref().unwrap())
                                .map(|ident: &Ident2| format_ident!("lhs_{}", ident))
                                .collect();
                            let rhs_names: Vec<Ident2> = rhs_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref().unwrap())
                                .map(|ident| format_ident!("delta_{}", ident))
                                .collect();
                            let field_assignments: TokenStream2 = lhs_variant
                                .fields().enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = field.name_ref()?;
                                    let dname = format_ident!("delta_{}", fname);
                                    let lfname = &lhs_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! { #fname: #lfname.clone(), });
                                    }
                                    Ok(quote! {
                                        #fname: match #dname.as_ref() {
                                            None => #lfname.clone(),
                                            Some(d) => #lfname.apply_delta(d)?,
                                        },
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                if let (
                                    Self::#lhs_variant_name {
                                        #( #field_names : #lhs_names, )*
                                    },
                                    Self::Delta::#rhs_variant_name {
                                        #( #field_names : #rhs_names, )*
                                    }
                                ) = (self, delta) {
                                    return Ok(Self::#lhs_variant_name {
                                        #field_assignments
                                    })
                                }
                            }
                        },
                        (StructVariant::TupleStruct, StructVariant::TupleStruct)
                            if lhs_variant_name == rhs_variant_name =>
                        {
                            let field_count = field_types.len();
                            let lhs_names: Vec<Ident2> = (0 .. field_count)
                                .map(|ident| format_ident!("lhs_{}", ident))
                                .collect();
                            let rhs_names: Vec<Ident2> = (0 .. field_count)
                                .map(|ident| format_ident!("delta_{}", ident))
                                .collect();
                            let field_assignments: TokenStream2 = lhs_variant
                                .fields().enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = &lhs_names[fidx];
                                    let dname = &rhs_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! { #fname.clone(), });
                                    }
                                    Ok(quote! {
                                        match #dname.as_ref() {
                                            None => #fname.clone(),
                                            Some(d) => #fname.apply_delta(d)?,
                                        },
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                if let (
                                    Self::#lhs_variant_name(#(#lhs_names),*),
                                    Self::Delta::#rhs_variant_name(#(#rhs_names),*)
                                ) = (self, delta) {
                                    return Ok(Self::#lhs_variant_name(
                                        #field_assignments
                                    ))
                                }
                            }
                        },
                        (_, StructVariant::UnitStruct)
                            if lhs_variant_name == rhs_variant_name => quote! {
                                if let (
                                    Self::#lhs_variant_name,
                                    Self::Delta::#rhs_variant_name
                                ) = (self, delta) {
                                    return Ok(Self::#lhs_variant_name)
                                }
                            },
                        _ => quote! { },
                    });
                }
                for enum_variant in variants.iter() {
                    let struct_variant = enum_variant.struct_variant;
                    let variant_name = &enum_variant.name;
                    let field_types: Vec<&Type> = enum_variant.fields()
                        .map(|field: &FieldDesc| field.type_ref())
                        .collect();
                    apply_delta_tokens.extend(match struct_variant {
                        StructVariant::NamedStruct => {
                            let field_names: Vec<&Ident2> = enum_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref())
                                .collect::<DeriveResult<_>>()?;
                            let field_assignments: TokenStream2 = enum_variant.fields()
                                .enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let lfname = &field_names[fidx];
                                    let ftype = &field_types[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            #lfname: Default::default(),
                                        });
                                    }
                                    Ok(quote! {
                                        #lfname: match #lfname {
                                            Some(d) => <#ftype>::from_delta(d.clone())?,
                                            None => return Err(DeltaError::ExpectedValue)?,
                                        },
                                    })
                                })
                                .collect::<DeriveResult<_>>()?;
                            quote! {
                                if let Self::Delta::#variant_name {
                                    #(ref #field_names),*
                                } = delta {
                                    use deltoid::{DeltaError, FromDelta};
                                    return Ok(Self::#variant_name {
                                        #field_assignments
                                    })
                                }
                            }
                        },
                        StructVariant::TupleStruct => {
                            let field_names: Vec<Ident> = (0 .. field_types.len())
                                .map(|token| format_ident!("field{}", token))
                                .collect();
                            let field_assignments: TokenStream2 = enum_variant.fields()
                                .enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let lfname = &field_names[fidx];
                                    let ftype = &field_types[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! { Default::default(), });
                                    }
                                    Ok(quote! {
                                        match #lfname {
                                            Some(d) => <#ftype>::from_delta(d.clone())?,
                                            None => return Err(DeltaError::ExpectedValue)?,
                                        },
                                    })
                                })
                                .collect::<DeriveResult<_>>()?;
                            quote! {
                                if let Self::Delta::#variant_name(
                                    #(ref #field_names),*
                                ) = delta {
                                    use deltoid::{DeltaError, FromDelta};
                                    return Ok(Self::#variant_name(
                                        #field_assignments
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
                for (lhs_variant, rhs_variant) in iproduct!(
                    variants.iter(), variants.iter()
                ) {
                    let lhs_struct_variant = lhs_variant.struct_variant;
                    let rhs_struct_variant = rhs_variant.struct_variant;
                    let lhs_variant_name = &lhs_variant.name;
                    let rhs_variant_name = &rhs_variant.name;
                    let field_types: Vec<&Type> = lhs_variant.fields()
                        .map(|field: &FieldDesc| field.type_ref())
                        .collect();
                    delta_tokens.extend(match (lhs_struct_variant, rhs_struct_variant) {
                        (StructVariant::NamedStruct, StructVariant::NamedStruct)
                            if lhs_variant_name == rhs_variant_name =>
                        {
                            let field_names: Vec<&Ident2> = lhs_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref())
                                .collect::<DeriveResult<_>>()?;
                            let lfield_names: Vec<Ident> = field_names.iter()
                                .map(|ident| format_ident!("lhs_{}", ident))
                                .collect();
                            let rfield_names: Vec<Ident> = field_names.iter()
                                .map(|ident| format_ident!("rhs_{}", ident))
                                .collect();
                            let field_assignments: TokenStream2 = lhs_variant
                                .fields().enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = field.name_ref()?;
                                    let lfname = &lfield_names[fidx];
                                    let rfname = &rfield_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            #fname: std::marker::PhantomData,
                                        });
                                    }
                                     Ok(quote! {
                                        #fname: if #lfname != #rfname {
                                            Some(#lfname.delta(&#rfname)?)
                                        } else {
                                            None
                                        },
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                if let (
                                    Self::#lhs_variant_name {
                                        #(#field_names : #lfield_names),*
                                    },
                                    Self::#rhs_variant_name {
                                        #(#field_names : #rfield_names),*
                                    },
                                ) = (self, rhs) {
                                    return Ok(Self::Delta::#lhs_variant_name {
                                        #field_assignments
                                    });
                                }
                            }
                        },
                        (StructVariant::TupleStruct, StructVariant::TupleStruct)
                            if lhs_variant_name == rhs_variant_name =>
                        {
                            let field_count = field_types.len();
                            let lfield_names: Vec<Ident> = (0 .. field_count)
                                .map(|token| format_ident!("lhs_{}", token))
                                .collect();
                            let rfield_names: Vec<Ident> = (0 .. field_count)
                                .map(|token| format_ident!("rhs_{}", token))
                                .collect();
                            let field_assignments: TokenStream2 = lhs_variant
                                .fields().enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let lfname = &lfield_names[fidx];
                                    let rfname = &rfield_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            // #lfname.clone().into_delta(),
                                            std::marker::PhantomData,
                                        });
                                    }
                                    Ok(quote! {
                                        if #lfname != #rfname {
                                            Some(#lfname.delta(&#rfname)?)
                                        } else {
                                            None
                                        },
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                if let (
                                    Self::#lhs_variant_name( #(#lfield_names),* ),
                                    Self::#rhs_variant_name( #(#rfield_names),* ),
                                ) = (self, rhs) {
                                    return Ok(Self::Delta::#lhs_variant_name(
                                        #field_assignments
                                    ));
                                }
                            }
                        },
                        (_, StructVariant::UnitStruct)
                            if lhs_variant_name == rhs_variant_name =>
                        {
                            quote! {
                                if let (
                                    Self::#lhs_variant_name,
                                    Self::#rhs_variant_name
                                ) = (self, rhs) {
                                    return Ok(Self::Delta::#rhs_variant_name);
                                }
                            }
                        },
                        _ => quote! { },
                    });
                }
                for enum_variant in variants.iter() {
                    let struct_variant = enum_variant.struct_variant;
                    let variant_name = &enum_variant.name;
                    let field_types: Vec<&Type> = enum_variant.fields()
                        .map(|field: &FieldDesc| field.type_ref())
                        .collect();
                    delta_tokens.extend(match struct_variant {
                        StructVariant::NamedStruct => {
                            let field_names: Vec<&Ident2> = enum_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref())
                                .collect::<DeriveResult<_>>()?;
                            let field_assignments: TokenStream2 = enum_variant
                                .fields().enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = &field_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            #fname: std::marker::PhantomData,
                                        });
                                    }
                                    Ok(quote! {
                                        #fname: Some(
                                            #fname.clone().into_delta()?
                                        ),
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                if let Self::#variant_name { #(#field_names),* } = rhs {
                                    return Ok(Self::Delta::#variant_name {
                                        #field_assignments
                                    });
                                }
                            }
                        },
                        StructVariant::TupleStruct => {
                            let field_count = field_types.len();
                            let field_names: Vec<Ident> = (0 .. field_count)
                                .map(|token| format_ident!("field{}", token))
                                .collect();
                            let field_assignments: TokenStream2 = enum_variant
                                .fields().enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = &field_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            std::marker::PhantomData,
                                        });
                                    }
                                    Ok(quote! {
                                        Some(#fname.clone().into_delta()?),
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                if let Self::#variant_name( #(#field_names),* ) = rhs {
                                    return Ok(Self::Delta::#variant_name(
                                        #field_assignments
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
                quote! {
                    impl<#input_type_param_decls> deltoid::DeltaOps
                        for #type_name<#input_type_params>
                        #where_clause
                    {
                        type Delta = #delta_type_name<#input_type_params>;

                        #[allow(unused)]
                        fn apply_delta(&self, delta: &Self::Delta) ->
                            deltoid::DeltaResult<Self> {
                            #apply_delta_tokens
                            deltoid::bug_detected!()
                        }

                        #[allow(unused)]
                        fn delta(&self, rhs: &Self) ->
                            deltoid::DeltaResult<Self::Delta>
                        {
                            use deltoid::IntoDelta;
                            #delta_tokens
                            deltoid::bug_detected!()
                        }
                    }
                }
            },
            Self::Union { .. } => {
                unimplemented!("Delta computation for unions is not supported.");
            },
        })
    }

    #[allow(non_snake_case)]
    pub fn define_FromDelta_impl(&self) -> DeriveResult<TokenStream2> {
        let accumulate_tokens = |total_result, elt_result| {
            let (total, elt) = (total_result?, elt_result?);
            Ok(quote! { #total  #elt })
        };
        Ok(match self {
            Self::Struct {
                struct_variant,
                type_name,
                delta_name: delta_type_name,
                input_type_params,
                input_type_param_decls,
                fields,
                deltaops_trait_impl_where_clause: where_clause,
                ..
            } => {
                let mut match_body = TokenStream2::new();
                match_body.extend(match struct_variant {
                    StructVariant::NamedStruct => {
                        let field_names: Vec<_> = fields.iter()
                            .map(|field: &FieldDesc| field.name_ref())
                            .collect::<DeriveResult<_>>()?;
                        let field_assignments: TokenStream2 = fields.iter()
                            .map(|field: &FieldDesc| {
                                let fname = field.name_ref()?;
                                let ftype = field.type_ref();
                                if field.ignore_field() {
                                    return Ok(quote! {
                                        #fname: Default::default(),
                                    });
                                }
                                Ok(quote! {
                                    #fname: <#ftype>::from_delta(
                                        #fname.ok_or(DeltaError::ExpectedValue)?
                                    )?,
                                })
                            })
                            .fold(Ok(quote!{ }), accumulate_tokens)?;
                        quote! {
                            #delta_type_name { #( #field_names ),* } => {
                                Self { #field_assignments }
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
                        let field_assignments: TokenStream2 = fields.iter()
                            .enumerate()
                            .map(|(fidx, field): (usize, &FieldDesc)| {
                                let fname = &field_names[fidx];
                                let ftype = field.type_ref();
                                if field.ignore_field() {
                                    return Ok(quote! { Default::default(), });
                                }
                                Ok(quote! {
                                    <#ftype>::from_delta(
                                        #fname.ok_or(DeltaError::ExpectedValue)?
                                    )?,
                                })
                            })
                            .fold(Ok(quote!{ }), accumulate_tokens)?;
                        quote! {
                            #delta_type_name( #( #field_names ),* ) => {
                                Self( #field_assignments )
                            },
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        #delta_type_name => Self,
                    },
                });
                quote! {
                    impl<#input_type_param_decls> deltoid::FromDelta
                        for #type_name<#input_type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn from_delta(
                            delta: <Self as deltoid::DeltaOps>::Delta
                        ) -> deltoid::DeltaResult<Self> {
                            use deltoid::DeltaError;
                            Ok(match delta {
                                #match_body
                            })
                        }
                    }
                }
            },
            Self::Enum {
                type_name,
                delta_name: delta_type_name,
                variants,
                input_type_params,
                input_type_param_decls,
                deltaops_trait_impl_where_clause: where_clause,
                ..
            } => {
                let mut match_body = TokenStream2::new();
                for enum_variant in variants.iter() {
                    let struct_variant = enum_variant.struct_variant;
                    let variant_name = &enum_variant.name;
                    match_body.extend(match struct_variant {
                        StructVariant::NamedStruct => {{
                            let field_names: Vec<_> = enum_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref())
                                .collect::<DeriveResult<_>>()?;
                            let field_assignments: TokenStream2 = enum_variant.fields()
                                .map(|field: &FieldDesc| {
                                    let fname = field.name_ref()?;
                                    let ftype = field.type_ref();
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            #fname: Default::default(),
                                        });
                                    }
                                    Ok(quote! {
                                        #fname: <#ftype>::from_delta(
                                            #fname.ok_or(DeltaError::ExpectedValue)?
                                        )?,
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                #delta_type_name::#variant_name {
                                    #( #field_names ),*
                                } => {
                                    Self::#variant_name { #field_assignments }
                                },
                            }
                        }},
                        StructVariant::TupleStruct => {{
                            let field_types: Vec<_> = enum_variant.fields()
                                .map(|field: &FieldDesc| field.type_ref())
                                .collect();
                            let field_count = field_types.len();
                            let field_names: Vec<Ident> = (0 .. field_count)
                                .map(|token| format_ident!("field{}", token))
                                .collect();
                            let field_assignments: TokenStream2 = enum_variant.fields()
                                .enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = &field_names[fidx];
                                    let ftype = field.type_ref();
                                    if field.ignore_field() {
                                        return Ok(quote! { Default::default(), });
                                    }
                                    Ok(quote! {
                                        <#ftype>::from_delta(
                                            #fname.ok_or(DeltaError::ExpectedValue)?
                                        )?,
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                #delta_type_name::#variant_name(
                                    #( #field_names ),*
                                ) => {
                                    Self::#variant_name( #field_assignments )
                                },
                            }
                        }},
                        StructVariant::UnitStruct => quote! {
                            #delta_type_name::#variant_name => {
                                Self::#variant_name
                            },
                        },
                    });
                }
                quote! {
                    impl<#input_type_param_decls> deltoid::FromDelta
                        for #type_name<#input_type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn from_delta(
                            delta: <Self as deltoid::DeltaOps>::Delta
                        ) -> deltoid::DeltaResult<Self> {
                            use deltoid::DeltaError;
                            Ok(match delta {
                                #match_body
                            })
                        }
                    }
                }
            },
            Self::Union { .. } => {
                unimplemented!("Delta computation for unions is not supported.");
            },
        })
    }

    #[allow(non_snake_case)]
    pub fn define_IntoDelta_impl(&self) -> DeriveResult<TokenStream2> {
        let accumulate_tokens = |total_result, elt_result| {
            let (total, elt) = (total_result?, elt_result?);
            Ok(quote! { #total  #elt })
        };
        Ok(match self {
            Self::Struct {
                struct_variant,
                type_name,
                delta_name: delta_type_name,
                input_type_params,
                input_type_param_decls,
                fields,
                deltaops_trait_impl_where_clause: where_clause,
                ..
            } => {
                let mut match_body = TokenStream2::new();
                match_body.extend(match struct_variant {
                    StructVariant::NamedStruct => {
                        let field_names: Vec<_> = fields.iter()
                            .map(|field: &FieldDesc| field.name_ref())
                            .collect::<DeriveResult<_>>()?;
                        let field_assignments: TokenStream2 = fields.iter()
                            .map(|field: &FieldDesc| {
                                let fname = field.name_ref()?;
                                if field.ignore_field() {
                                    return Ok(quote! {
                                        #fname: std::marker::PhantomData,
                                    });
                                }
                                Ok(quote! {
                                    #fname: Some(#fname.into_delta()?),
                                })
                            })
                            .fold(Ok(quote!{ }), accumulate_tokens)?;
                        quote! {
                            Self { #( #field_names, )* .. } => {
                                #delta_type_name { #field_assignments }
                            },
                        }
                    },
                    StructVariant::TupleStruct => {
                        let field_count = fields.count_fields();
                        let field_names: Vec<Ident> = (0 .. field_count)
                            .map(|token| format_ident!("field_{}", token))
                            .collect();
                        let field_assignments: TokenStream2 = fields.iter()
                            .enumerate()
                            .map(|(fidx, field): (usize, &FieldDesc)| {
                                let fname = &field_names[fidx];
                                if field.ignore_field() {
                                    return Ok(quote! {
                                        std::marker::PhantomData,
                                    });
                                }
                                Ok(quote! {
                                    Some(#fname.into_delta()?),
                                })
                            })
                            .fold(Ok(quote!{ }), accumulate_tokens)?;
                        quote! {
                            Self(#( #field_names, )* ..) => {
                                #delta_type_name( #field_assignments )
                            },
                        }
                    },
                    StructVariant::UnitStruct => quote! {
                        Self => #delta_type_name,
                    },
                });
                quote! {
                    impl<#input_type_param_decls> deltoid::IntoDelta
                        for #type_name<#input_type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn into_delta(self) -> deltoid::DeltaResult<
                            <Self as deltoid::DeltaOps>::Delta
                        > {
                            use deltoid::IntoDelta;
                            Ok(match self {
                                #match_body
                            })
                        }
                    }
                }
            },
            Self::Enum {
                type_name,
                delta_name: delta_type_name,
                variants,
                input_type_params,
                input_type_param_decls,
                deltaops_trait_impl_where_clause: where_clause,
                ..
            } => {
                let mut match_body = TokenStream2::new();
                for enum_variant in variants.iter() {
                    let struct_variant = enum_variant.struct_variant;
                    let variant_name = &enum_variant.name;
                    match_body.extend(match struct_variant {
                        StructVariant::NamedStruct => {{
                            let field_names: Vec<_> = enum_variant.fields()
                                .map(|field: &FieldDesc| field.name_ref())
                                .collect::<DeriveResult<_>>()?;
                            let field_assignments: TokenStream2 = enum_variant.fields()
                                .map(|field: &FieldDesc| {
                                    let fname = field.name_ref()?;
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            #fname: std::marker::PhantomData,
                                        });
                                    }
                                    Ok(quote! {
                                        #fname: Some(#fname.into_delta()?),
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                Self::#variant_name { #( #field_names, )* .. } => {
                                    #delta_type_name::#variant_name {
                                        #field_assignments
                                    }
                                },
                            }
                        }},
                        StructVariant::TupleStruct => {{
                            let field_count = enum_variant.fields().count();
                            let field_names: Vec<Ident> = (0 .. field_count)
                                .map(|token| format_ident!("field_{}", token))
                                .collect();
                            let field_assignments: TokenStream2 = enum_variant.fields()
                                .enumerate()
                                .map(|(fidx, field): (usize, &FieldDesc)| {
                                    let fname = &field_names[fidx];
                                    if field.ignore_field() {
                                        return Ok(quote! {
                                            std::marker::PhantomData,
                                        });
                                    }
                                    Ok(quote! {
                                        Some(#fname.into_delta()?),
                                    })
                                })
                                .fold(Ok(quote!{ }), accumulate_tokens)?;
                            quote! {
                                Self::#variant_name( #( #field_names, )* ..) => {
                                    #delta_type_name::#variant_name(
                                        #field_assignments
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
                quote! {
                    impl<#input_type_param_decls> deltoid::IntoDelta
                        for #type_name<#input_type_params>
                        #where_clause
                    {
                        #[allow(unused)]
                        fn into_delta(self) -> deltoid::DeltaResult<
                            <Self as deltoid::DeltaOps>::Delta
                        > {
                            use deltoid::IntoDelta;
                            Ok(match self {
                                #match_body
                            })
                        }
                    }
                }
            },
            Self::Union { .. } => {
                unimplemented!("Delta computation for unions is not supported.");
            },
        })
    }

    fn define_delta_struct(
        struct_variant: StructVariant,
        delta_struct_name: &Ident,
        fields: &FieldsDesc,
        type_param_decls: &Punctuated<GenericParam, Comma>,
        where_clause: &WhereClause,
    ) -> DeriveResult<TokenStream2> {
        let field_types: Vec<TokenStream2> = fields.iter()
            .map(|field: &FieldDesc| field.type_tokens())
            .collect();
        Ok(match struct_variant {
            StructVariant::NamedStruct => {
                let field_names: Vec<&Ident2> = fields.iter()
                    .map(|field: &FieldDesc| field.name_ref())
                    .collect::<DeriveResult<_>>()?;
                quote! {
                    #[derive(Debug, PartialEq, Clone)]
                    #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
                    pub struct #delta_struct_name<#type_param_decls>
                        #where_clause
                    {
                        #( pub(self) #field_names: #field_types, )*
                    }
                }
            },
            StructVariant::TupleStruct => quote! {
                #[derive(Debug, PartialEq, Clone)]
                #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
                pub struct #delta_struct_name<#type_param_decls> (
                    #( pub(self) #field_types, )*
                ) #where_clause ;
            },
            StructVariant::UnitStruct => quote! {
                #[derive(Debug, PartialEq, Clone)]
                #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
                pub struct #delta_struct_name<#type_param_decls>
                    #where_clause ;
            },
        })
    }

    fn define_delta_enum(
        delta_enum_name: &Ident,
        enum_variants: &EnumVariantsDesc,
        type_param_decls: &Punctuated<GenericParam, Comma>,
        where_clause: &WhereClause,
    ) -> DeriveResult<TokenStream2> {
        let enum_body: TokenStream2 = enum_variants.iter()
            .map(|enum_variant: &EnumVariantDesc| -> DeriveResult<_> {
                let variant_name = &enum_variant.name;
                let field_types: Vec<TokenStream2> = enum_variant.fields()
                    .map(|field: &FieldDesc| field.type_tokens())
                    .collect();
                Ok(match enum_variant.struct_variant {
                    StructVariant::NamedStruct =>  {
                        let field_names: Vec<&Ident2> = enum_variant.fields()
                            .map(|field: &FieldDesc| field.name_ref())
                            .collect::<DeriveResult<_>>()?;
                        quote! {
                            #variant_name { #( #field_names: #field_types, )* },
                        }
                    },
                    StructVariant::TupleStruct =>  quote! {
                        #variant_name( #( #field_types, )* ),
                    },
                    StructVariant::UnitStruct =>  quote! { #variant_name, },
                })
            })
            .collect::<DeriveResult<_>>()?;
        Ok(quote! {
            #[derive(Debug, PartialEq, Clone)]
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub enum #delta_enum_name<#type_param_decls> #where_clause {
                #enum_body
            }
        })
    }
}




#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructVariant {
    /// A "named struct" i.e. a struct with named fields
    NamedStruct,
    /// A tuple struct i.e. unnamed/positional fields
    TupleStruct,
    /// A unit struct i.e. no fields at all
    UnitStruct,
}


#[derive(Clone, Debug)]
pub struct EnumVariantsDesc(Vec<EnumVariantDesc>);

impl EnumVariantsDesc {
    pub fn new() -> Self { Self(vec![]) }

    pub fn add_variant(&mut self, variant: EnumVariantDesc) {
        self.0.push(variant);
    }

    pub fn iter(&self) -> impl Iterator<Item = &EnumVariantDesc> + Clone {
        self.0.iter()
    }
}


#[derive(Clone, Debug)]
pub struct EnumVariantDesc {
    struct_variant: StructVariant,
    name: Ident2,
    fields: FieldsDesc,
}

impl EnumVariantDesc {
    pub fn new(name: &Ident2) -> Self {
        Self {
            struct_variant: StructVariant::UnitStruct,
            name: name.clone(),
            fields: FieldsDesc::new(),
        }
    }

    pub fn add_field(&mut self, field: FieldDesc) { self.fields.add(field); }

    pub fn fields(&self) -> impl Iterator<Item = &FieldDesc> {
        self.fields.iter()
    }
}




#[derive(Clone, Debug)]
pub struct FieldsDesc(Vec<FieldDesc>);

impl FieldsDesc {
    pub fn new() -> Self { Self(vec![]) }

    pub fn add(&mut self, field: FieldDesc) { self.0.push(field); }

    pub fn iter(&self) -> impl Iterator<Item = &FieldDesc> { self.0.iter() }

    pub fn count_fields(&self) -> usize { self.iter().count() }
}


#[derive(Clone, Debug)]
/// A description of a field that's part of a `struct` or an `enum`.
pub enum FieldDesc {
    /// A field that's part of a named struct
    Named {
        name: Ident2,
        ty: Type,
        ignore_field: bool,
    },
    /// A field that's part of a tuple struct
    Positional {
        position: Literal2,
        ty: Type,
        ignore_field: bool,
    }
}

#[allow(non_snake_case)]
impl FieldDesc {
    pub fn name_ref(&self) -> DeriveResult<&Ident2> {
        match self {
            Self::Named { name, .. } => Ok(name),
            Self::Positional { .. } => Err(DeriveError::ExpectedNamedField),
        }
    }

    pub fn pos_ref(&self) -> DeriveResult<&Literal2> {
        match self {
            Self::Named { .. } => Err(DeriveError::ExpectedPositionalField),
            Self::Positional { position, .. } => Ok(position),
        }
    }

    pub fn type_ref(&self) -> &Type {
        match self {
            Self::Named { ty, .. } => ty,
            Self::Positional { ty, .. } => ty,
        }
    }

    pub fn type_tokens(&self) -> TokenStream2 {
        let ty: &Type = self.type_ref();
        if self.ignore_field() {
            quote! { std::marker::PhantomData<#ty> }
        } else {
            quote! {
                Option<<#ty as deltoid::DeltaOps>::Delta>
            }
        }
    }

    pub fn ignore_field(&self) -> bool {
        match self {
            Self::Named { ignore_field, .. } => *ignore_field,
            Self::Positional { ignore_field, .. } => *ignore_field,
        }
    }
}



/// A `field` in the input struct or input enum variant
/// is marked as #[delta(ignore_field)].
pub fn ignore_field(field: &Field) -> bool {
    use proc_macro2::{Delimiter as Delimiter2, TokenTree as TokenTree2};
    let mut ignore = false;
    for attr in field.attrs.iter() {
        let attr_segments: Vec<String> = attr.path.segments.iter()
            .map(|path_segment| format!("{}", path_segment.ident))
            .collect();
        let is_delta_attr = attr_segments == &["delta"];
        let arg_tokens_iter = attr.tokens.clone().into_iter().next();
        const DELIM: Delimiter2 = Delimiter2::Parenthesis;
        let arg_is_ignore_field = match arg_tokens_iter {
            Some(TokenTree2::Group(g)) if g.delimiter() == DELIM => {
                let tokens: Vec<String> = g.stream().clone().into_iter()
                    .map(|tt| format!("{}", tt))
                    .collect();
                tokens == &["ignore_field"]
            },
            _ => false,
        };
        ignore = ignore || is_delta_attr && arg_is_ignore_field
    }
    ignore
}


pub fn empty_where_clause() -> WhereClause {
    WhereClause {
        where_token: Token![where](Span2::call_site()),
        predicates: Punctuated::new(),
    }
}
