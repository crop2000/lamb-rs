fn main() {
    #[cfg(feature = "faust-rebuild")]
    {
        use faust_build::{builder::FaustBuilder, code_option::CodeOption};
        use std::{env, path::Path};

        fn lamb_build_settings(a: &mut FaustBuilder) {
            let json_out_dir =
                env::var_os("OUT_DIR").expect("OUT_DIR environement variable is not set.");
            let target_json_folder = Path::new(&json_out_dir).to_path_buf();
            a.set_code_option(CodeOption::Double);
            a.set_code_option(CodeOption::InPlace);
            a.set_code_option(CodeOption::NoFaustDsp);
            a.add_code_gen_fun(faust_build::generate::create_inplace_vec_trait);
            a.add_derive("default_boxed::DefaultBoxed");
            a.set_json_folder(target_json_folder);
        }

        println!("cargo:rerun-if-changed=dsp");

        let mut builder = faust_ui_build::file_with_ui("dsp/lamb-rs-48k.dsp", "src/dsp_48k.rs");
        lamb_build_settings(&mut builder);
        builder.build();

        let mut builder = faust_ui_build::file_with_ui("dsp/lamb-rs-96k.dsp", "src/dsp_96k.rs");
        lamb_build_settings(&mut builder);
        builder.build();

        let mut builder = faust_ui_build::file_with_ui("dsp/lamb-rs-192k.dsp", "src/dsp_192k.rs");
        lamb_build_settings(&mut builder);
        builder.build();
    }
}
