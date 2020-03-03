extern crate proc_macro;

#[macro_use] mod error;
mod desc;

use crate::desc::{UserDefinedTypeDesc};
use crate::error::{DeriveError, DeriveResult};
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote};
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(Delta, attributes(delta))]
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
    let type_desc = UserDefinedTypeDesc::parse(&input)?;
    let delta_type_definition         = type_desc.define_delta_type()?;
    let impl_Deltoid_for_input_type   = type_desc.define_Deltoid_impl()?;
    let impl_IntoDelta_for_input_type = type_desc.define_IntoDelta_impl()?;
    let impl_FromDelta_for_input_type = type_desc.define_FromDelta_impl()?;
    let output: TokenStream2 = quote! {
        #delta_type_definition
        #impl_Deltoid_for_input_type
        #impl_IntoDelta_for_input_type
        #impl_FromDelta_for_input_type
    };

    // print_generated_code(
    //     &delta_type_definition,
    //     &impl_Deltoid_for_input_type,
    //     &impl_FromDelta_for_input_type,
    //     &impl_IntoDelta_for_input_type
    // );

    // write_generated_code_to_file(
    //     &delta_type_definition,
    //     &impl_Deltoid_for_input_type,
    //     &impl_FromDelta_for_input_type,
    //     &impl_IntoDelta_for_input_type
    // );

    Ok(output)
}


#[allow(unused, non_snake_case)]
fn print_generated_code(
    delta_type_definition: &TokenStream2,
    impl_Deltoid_for_input_type: &TokenStream2,
    impl_FromDelta_for_input_type: &TokenStream2,
    impl_IntoDelta_for_input_type: &TokenStream2,
) {
    println!("{}\n", delta_type_definition);
    println!("{}\n", impl_Deltoid_for_input_type);
    println!("{}\n", impl_FromDelta_for_input_type);
    println!("{}\n", impl_IntoDelta_for_input_type);
    println!("\n\n\n\n");
}


#[allow(unused, non_snake_case)]
fn write_generated_code_to_file(
    delta_type_definition: &TokenStream2,
    impl_Deltoid_for_input_type: &TokenStream2,
    impl_FromDelta_for_input_type: &TokenStream2,
    impl_IntoDelta_for_input_type: &TokenStream2,
) {
    use std::io::Write;
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("/home/j/dev/deltoid/floof.rs")
        .expect("Failed to write floof.rs");

    file.write_all(format!("{}", delta_type_definition).as_bytes())
        .expect("Failed to write delta_type_definition");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_Deltoid_for_input_type).as_bytes())
        .expect("Failed to write impl_Deltoid_for_input_type");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_FromDelta_for_input_type).as_bytes())
        .expect("Failed to write impl_FromDelta_for_input_type");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_IntoDelta_for_input_type).as_bytes())
        .expect("Failed to write impl_IntoDelta_for_input_type");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.flush().expect("Failed to flush floof.rs");
    std::process::Command::new("rustfmt")
        .args(&["--emit-files", "--edition 2018", "/home/j/dev/deltoid/floof.rs"])
        .output()
        .expect("failed to execute rustfmt;");
}
