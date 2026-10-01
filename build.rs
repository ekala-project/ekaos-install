fn main() {
    // Embed RPATHs for native libraries so the binary can find them at runtime.
    // This is necessary on NixOS where libraries live in unique /nix/store paths.
    for lib in &["xcb", "xkbcommon", "xkbcommon-x11", "freetype2"] {
        if let Ok(output) = std::process::Command::new("pkg-config")
            .args(["--libs-only-L", lib])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for token in stdout.split_whitespace() {
                if let Some(path) = token.strip_prefix("-L") {
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{path}");
                }
            }
        }
    }
}
