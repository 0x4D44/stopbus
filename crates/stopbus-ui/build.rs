use std::env;
use std::fs;
use std::path::Path;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let rc_path = Path::new("resources/stopbus.rc");
    if !rc_path.exists() {
        panic!("missing {}", rc_path.display());
    }

    let _ = embed_resource::compile(rc_path, embed_resource::NONE);

    println!("cargo:rerun-if-changed={}", rc_path.display());
    let cards_dir = rc_path.parent().unwrap().join("cards");
    for entry in fs::read_dir(&cards_dir).expect("missing cards directory") {
        let path = entry.expect("read_dir error").path();
        if path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("bmp"))
            .unwrap_or(false)
        {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    println!(
        "cargo:rerun-if-changed={}",
        Path::new("../../MD.ICO").display()
    );
}
