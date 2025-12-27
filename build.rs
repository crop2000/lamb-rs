#[cfg(feature = "faust-rebuild")]
use faust_build::code_option::CodeOption::Double;

fn main() {
    println!("cargo:rerun-if-changed=dsp");

    #[cfg(feature = "faust-rebuild")]
    {
        let mut a = faust_build::default::for_file("dsp/lamb-rs-48k.dsp", "src/dsp_48k.rs");
        a.set_code_option(Double);
        a.build();

        let mut a = faust_build::default::for_file("dsp/lamb-rs-96k.dsp", "src/dsp_96k.rs");
        a.set_code_option(Double);
        a.build();

        let mut a = faust_build::default::for_file("dsp/lamb-rs-192k.dsp", "src/dsp_192k.rs");
        a.set_code_option(Double);
        a.build();
    }
}
