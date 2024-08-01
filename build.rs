fn main() {
    println!("cargo:rustc-cfg=oom_handle");
    println!("cargo::rustc-check-cfg=cfg(oom_handle)");
}
