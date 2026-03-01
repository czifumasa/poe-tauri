fn main() {
    tauri_build::build();

    println!("cargo:rustc-check-cfg=cfg(linux_bsd_target_os)");
    println!("cargo:rustc-check-cfg=cfg(windows_target_os)");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let linux_bsd_target_os = matches!(
        target_os.as_str(),
        "linux" | "dragonfly" | "freebsd" | "netbsd" | "openbsd"
    );

    if linux_bsd_target_os {
        println!("cargo:rustc-cfg=linux_bsd_target_os");
    }

    if target_os == "windows" {
        println!("cargo:rustc-cfg=windows_target_os");
    }
}
