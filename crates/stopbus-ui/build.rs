use std::env;
use std::fs::{self, File};
use std::io::Read;
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
            assert_modern_bitmap(&path);
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    println!(
        "cargo:rerun-if-changed={}",
        Path::new("../../MD.ICO").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        Path::new("../../assets/original-icons/icon_001.ico").display()
    );
}

fn assert_modern_bitmap(path: &Path) {
    let mut header = [0u8; 18];
    let mut file =
        File::open(path).unwrap_or_else(|err| panic!("failed to open {}: {}", path.display(), err));
    file.read_exact(&mut header)
        .unwrap_or_else(|err| panic!("failed to read header from {}: {}", path.display(), err));
    if &header[0..2] != b"BM" {
        panic!("{} is not a BMP file", path.display());
    }
    let dib_size = u32::from_le_bytes([header[14], header[15], header[16], header[17]]);
    if dib_size < 40 {
        panic!(
            "{} still uses an OS/2 BMP header ({} bytes). Run python tools/extract_cards.py {} before building.",
            path.display(),
            dib_size,
            path.display()
        );
    }
}
