"""Check the exact JS and serialized maps written by both versions."""

import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1]) / "tmp/project-output"
report = []
for case in sorted(root.iterdir()):
    def manifest(variant):
        directory = case / variant
        return {str(p.relative_to(directory)): (p.stat().st_size, hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(directory.rglob("*")) if p.is_file()}

    baseline = manifest("baseline")
    candidate = manifest("candidate")
    assert baseline and baseline == candidate, (case.name, baseline, candidate)
    mapped = case.name.endswith("_map") and not case.name.endswith("no_map")
    maps = list((case / "baseline").rglob("*.map"))
    assert bool(maps) == mapped, case
    for path in maps:
        source_map = json.loads(path.read_text())
        assert source_map["version"] == 3 and source_map["mappings"]
        assert len(source_map["sources"]) > 100
    report.append({"case": case.name, "identical": True, "files": baseline})
print(json.dumps(report, indent=2))
