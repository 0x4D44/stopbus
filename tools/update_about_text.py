from pathlib import Path
path = Path('crates/stopbus-ui/src/main.rs')
text = path.read_text()
text = text.replace(
    "const LICENSE_NAME: &str = \"This program is ShareWare.\";",
    "const LICENSE_NAME: &str = \"Rust modernization build.\";"
)
text = text.replace(
    "const LICENSE_COMPANY: &str =\n    \"If you like it then send 5 pounds (UK Sterling) or $10 (US dollars) to:\";",
    "const LICENSE_COMPANY: &str = \"Maintained by the community.\";"
)
text = text.replace(
    "const LICENSE_ADDRESS: &str =\n    \"Martin G Davidson\\r\\nHertford College\\r\\nOxford\\r\\nOX1 3BW\\r\\nUnited Kingdom\";",
    "const LICENSE_ADDRESS: &str = \"\";"
)
# Update release text to remove old shareware date
text = text.replace(
    "PCWSTR(wide_string(\"Released: 09/05/1994\").as_ptr()),",
    "PCWSTR(wide_string(\"Modernization build: 2025-09-22\").as_ptr()),"
)
path.write_text(text)
