fn main() {
    let ac = autocfg::new();
    if ac.probe_expression("format!(\"{:e}\", 0_i32)") {
        if !(ac.probe_expression("format!(\"{:e}\", 0_i8)")
            && ac.probe_expression("format!(\"{:e}\", 0_i16)")
            && ac.probe_expression("format!(\"{:e}\", 0_i32)")
            && ac.probe_expression("format!(\"{:e}\", 0_i64)")
            && ac.probe_expression("format!(\"{:e}\", 0_i128)")
            && ac.probe_expression("format!(\"{:e}\", 0_u8)")
            && ac.probe_expression("format!(\"{:e}\", 0_u16)")
            && ac.probe_expression("format!(\"{:e}\", 0_u32)")
            && ac.probe_expression("format!(\"{:e}\", 0_u64)")
            && ac.probe_expression("format!(\"{:e}\", 0_u128)"))
        {
            panic!("Some integer types implement *Exp traits, but not others")
        }
        println!("cargo:rustc-cfg=has_int_exp_fmt");
    }

    autocfg::rerun_path(file!());
}
