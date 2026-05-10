fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/platform/macos/bluetooth.m")
            .compile("bluetooth_macos");
        println!("cargo:rustc-link-lib=framework=IOBluetooth");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
