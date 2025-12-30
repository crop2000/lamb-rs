#[cfg(feature = "faust-rebuild")]
use faust_build::code_option::CodeOption;

fn main() {
    println!("cargo:rerun-if-changed=dsp");

    #[cfg(feature = "faust-rebuild")]
    {
        let mut a = faust_ui_build::file_with_ui("dsp/lamb-rs-48k.dsp", "src/dsp_48k.rs");
        a.set_code_option(CodeOption::Double);
        // a.set_code_option(CodeOption::InPlace);
        a.set_code_option(CodeOption::NoFaustDsp);
        a.build();

        let mut a = faust_ui_build::file_with_ui("dsp/lamb-rs-96k.dsp", "src/dsp_96k.rs");
        a.set_code_option(CodeOption::Double);
        // a.set_code_option(CodeOption::InPlace);
        a.set_code_option(CodeOption::NoFaustDsp);
        a.build();

        let mut a = faust_ui_build::file_with_ui("dsp/lamb-rs-192k.dsp", "src/dsp_192k.rs");
        a.set_code_option(CodeOption::Double);
        // a.set_code_option(CodeOption::InPlace);
        a.set_code_option(CodeOption::NoFaustDsp);
        a.build();
    }
}
