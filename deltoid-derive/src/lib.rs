extern crate proc_macro;

#[macro_use] mod error;
mod gen;

use crate::error::{DeriveError, DeriveResult};
use crate::gen::InputType;
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
#[cfg(feature = "dump-expansions--unstable")]
use proc_macro2::{Ident as Ident2};
use quote::quote;
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
    let input_type: InputType = InputType::parse(&input)?;
    let delta_type_definition = input_type.define_delta_type()?;
    let impl_Debug            = input_type.define_Debug_impl()?;
    let impl_Core             = input_type.define_Core_impl()?;
    let impl_Apply            = input_type.define_Apply_impl()?;
    let impl_Delta            = input_type.define_Delta_impl()?;
    let impl_FromDelta        = input_type.define_FromDelta_impl()?;
    let impl_IntoDelta        = input_type.define_IntoDelta_impl()?;
    let output: TokenStream2 = quote! {
        #delta_type_definition
        #impl_Debug
        #impl_Core
        #impl_Apply
        #impl_Delta
        #impl_FromDelta
        #impl_IntoDelta
    };

    #[cfg(feature = "print-expansions--unstable")]
    print_generated_code(
        &delta_type_definition,
        &impl_Debug,
        &impl_Core,
        &impl_Apply,
        &impl_Delta,
        &impl_FromDelta,
        &impl_IntoDelta,
    );

    #[cfg(feature = "dump-expansions--unstable")]
    write_generated_code_to_file(
        input_type.type_name()?,
        &delta_type_definition,
        &impl_Debug,
        &impl_Core,
        &impl_Apply,
        &impl_Delta,
        &impl_FromDelta,
        &impl_IntoDelta,
    );

    Ok(output)
}

#[cfg(feature = "print-expansions--unstable")]
#[allow(unused, non_snake_case)]
fn print_generated_code(
    delta_type_definition: &TokenStream2,
    impl_Debug: &TokenStream2,
    impl_Core: &TokenStream2,
    impl_Apply: &TokenStream2,
    impl_Delta: &TokenStream2,
    impl_FromDelta: &TokenStream2,
    impl_IntoDelta: &TokenStream2,
) {
    println!("{}\n", delta_type_definition);
    println!("{}\n", impl_Debug);
    println!("{}\n", impl_Core);
    println!("{}\n", impl_Apply);
    println!("{}\n", impl_Delta);
    println!("{}\n", impl_FromDelta);
    println!("{}\n", impl_IntoDelta);
    println!("\n\n\n\n");
}

#[cfg(feature = "dump-expansions--unstable")]
#[allow(unused, non_snake_case)]
fn write_generated_code_to_file(
    type_name: &Ident2,
    delta_type_definition: &TokenStream2,
    impl_Debug: &TokenStream2,
    impl_Core: &TokenStream2,
    impl_Apply: &TokenStream2,
    impl_Delta: &TokenStream2,
    impl_FromDelta: &TokenStream2,
    impl_IntoDelta: &TokenStream2,
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

    file.write_all(format!("{}", impl_Debug).as_bytes())
        .expect("Failed to write impl_Debug");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_Core).as_bytes())
        .expect("Failed to write impl_Core");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_Apply).as_bytes())
        .expect("Failed to write impl_Apply");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_Delta).as_bytes())
        .expect("Failed to write impl_Delta");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_FromDelta).as_bytes())
        .expect("Failed to write impl_FromDelta");
    file.write_all("\n\n".as_bytes()).expect("Failed to write newlines");

    file.write_all(format!("{}", impl_IntoDelta).as_bytes())
        .expect("Failed to write impl_IntoDelta");
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
