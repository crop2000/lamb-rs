#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
    unused_crate_dependencies,
    clippy::unwrap_used
)]

#[cfg(feature = "faust-rebuild")]
use faust_build::code_option::CodeOption;

fn main() {
    println!("cargo:rerun-if-changed=dsp");

    #[cfg(feature = "faust-rebuild")]
    {
        let mut a = faust_ui_build::file_with_ui("dsp/lamb-rs-48k.dsp", "src/dsp_48k.rs");
        a.set_code_option(CodeOption::Double);
        a.set_code_option(CodeOption::InPlace);
        a.set_code_option(CodeOption::NoFaustDsp);
        a.get_architecture_mut_ref()
            .add_derive(quote::quote! {default_boxed::DefaultBoxed});
        a.build();

        let mut a = faust_ui_build::file_with_ui("dsp/lamb-rs-96k.dsp", "src/dsp_96k.rs");
        a.set_code_option(CodeOption::Double);
        a.set_code_option(CodeOption::InPlace);
        a.set_code_option(CodeOption::NoFaustDsp);
        a.get_architecture_mut_ref()
            .add_derive(quote::quote! {default_boxed::DefaultBoxed});
        a.build();

        let mut a = faust_ui_build::file_with_ui("dsp/lamb-rs-192k.dsp", "src/dsp_192k.rs");
        a.set_code_option(CodeOption::Double);
        a.set_code_option(CodeOption::InPlace);
        a.set_code_option(CodeOption::NoFaustDsp);
        a.get_architecture_mut_ref()
            .add_derive(quote::quote! {default_boxed::DefaultBoxed});
        a.build();
    }
}
