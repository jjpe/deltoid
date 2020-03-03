extern crate proc_macro;

#[macro_use] mod error;
mod desc;

use crate::desc::{UserDefinedTypeDesc};
use crate::error::{DeriveError, DeriveResult};
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
#[cfg(feature = "dump-expansions--unstable")]
use proc_macro2::{Ident as Ident2};
use quote::{quote};
#[cfg(feature = "dump-expansions--unstable")]
use std::fs::{remove_file, File, OpenOptions};
#[cfg(feature = "dump-expansions--unstable")]
use std::io::Write;
#[cfg(feature = "dump-expansions--unstable")]
use std::path::{Path, PathBuf};
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

    #[cfg(feature = "dump-expansions--unstable")]
    write_generated_code_to_file(
        type_desc.type_name(),
        &delta_type_definition,
        &impl_Deltoid_for_input_type,
        &impl_FromDelta_for_input_type,
        &impl_IntoDelta_for_input_type,
    );

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


#[cfg(feature = "dump-expansions--unstable")]
#[allow(unused, non_snake_case)]
fn write_generated_code_to_file(
    type_name: &Ident2,
    delta_type_definition: &TokenStream2,
    impl_Deltoid_for_input_type: &TokenStream2,
    impl_FromDelta_for_input_type: &TokenStream2,
    impl_IntoDelta_for_input_type: &TokenStream2,
) {
    let manifest_dir: &Path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expanded_dir: PathBuf = manifest_dir.join("expanded");
    let filename: PathBuf = expanded_dir.join(&format!("{}.rs", type_name));

    create_dir(&expanded_dir);
    let _ = remove_file(&filename);
    let mut file: File = open_file(&filename);
    println!("wrote {}", filename.display());

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

    file.flush().expect(&format!("Failed to flush {}", filename.display()));
    std::process::Command::new("rustfmt")
        .args(&[
            "--emit", "files", "--edition", "2018",
            &format!("{}", filename.display())
        ])
        .output()
        .expect("failed to execute rustfmt;");
}

#[cfg(feature = "dump-expansions--unstable")]
fn create_dir<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(path)
        .expect(&format!("Failed to create dir {}", path.display()));
}

#[cfg(feature = "dump-expansions--unstable")]
fn open_file<P: AsRef<Path>>(path: P) -> File {
    let path = path.as_ref();
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .truncate(true)
        .open(path)
        .expect(&format!("Failed to open {}", path.display()))
}
