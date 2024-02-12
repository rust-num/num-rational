fn main() {
    let ac = autocfg::new();
    if ac.probe_expression("format!(\"{:e}\", 0_isize)") {
        println!("cargo:rustc-cfg=has_int_exp_fmt");
    }

    let std = if ac.probe_sysroot_crate("std") {
        "std"
    } else {
        "core"
    };
    if ac.probe_path(&format!("{}::convert::TryFrom", std)) {
        autocfg::emit("has_try_from");
    }

    autocfg::rerun_path("build.rs");
}
