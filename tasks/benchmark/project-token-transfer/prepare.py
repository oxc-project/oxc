"""Patch dependencies and install the whole-project driver into a pinned Rolldown checkout."""

import pathlib
import shutil
import sys
import tomllib

oxc = pathlib.Path(sys.argv[1]).resolve()
rolldown = pathlib.Path(sys.argv[2]).resolve()
manifest = rolldown / "Cargo.toml"
text = manifest.read_text()
assert "[patch.crates-io]" not in text
text += '\n[patch.crates-io]\n'
for cargo in sorted((oxc / "crates").glob("*/Cargo.toml")):
    package = tomllib.loads(cargo.read_text())["package"]
    name = package["name"]
    text += f'{name} = {{ path = "{cargo.parent}" }}\n'
text += 'oxc_sourcemap = { git = "https://github.com/oxc-project/oxc-sourcemap.git", rev = "7375335803ca5c37007c09ed61e41386e1fe5e68" }\n'
manifest.write_text(text)
examples = rolldown / "crates/bench/examples"
examples.mkdir(exist_ok=True)
shutil.copyfile(pathlib.Path(__file__).with_name("project_build.rs"), examples / "project_build.rs")
