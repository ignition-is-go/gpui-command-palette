#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
main = (root / "examples/web/main.rs").read_text()
html = (root / "examples/web/index.html").read_text()
dist = root / "examples/web/dist/index.html"
assert 'include!("../demo.rs")' in main, "wasm host must compile the exact native demo source"
assert "<canvas" not in html.lower(), "GPUI, not host markup, must own the canvas"
assert dist.is_file(), "Trunk production output is missing"
built = dist.read_text().lower()
assert built.count("<canvas") == 0, "Trunk host must not synthesize a second canvas"
print("shared source and document-owned canvas host verified")
