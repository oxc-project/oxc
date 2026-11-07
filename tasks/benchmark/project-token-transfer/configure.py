"""Generate paired native and memory CodSpeed exec configurations."""

import json
import pathlib
import shlex
import sys

root = pathlib.Path(sys.argv[1]).resolve()
inputs = pathlib.Path(sys.argv[2]).resolve()
binaries = pathlib.Path(sys.argv[3]).resolve()
configs = root / "tasks/benchmark/project-token-transfer"
cases = [(p, m, True) for p in ("three", "rome") for m in (False, True)]
cases += [(p, False, False) for p in ("three", "rome")]

for mode in ("walltime", "memory"):
    benchmarks = []
    passes = ("baseline_1", "candidate_1", "candidate_2", "baseline_2") if mode == "walltime" else ("baseline", "candidate")
    for project, minify, maps in cases:
        if mode == "memory" and not maps:
            continue
        case = f'{project}_{"minified" if minify else "normal"}_{"map" if maps else "no_map"}'
        for label in passes:
            variant = label.split("_")[0]
            output = root / f"tmp/project-output/{case}/{variant}"
            args = [str(binaries / variant), str(inputs), project, str(minify).lower(), str(maps).lower(), str(output)]
            benchmarks.append({"name": f"project_tokens/{case}/{label}", "exec": shlex.join(args)})
    config = {"benchmarks": benchmarks}
    if mode == "walltime":
        config["options"] = {"warmup-time": "1s", "min-rounds": 20, "max-rounds": 20}
    (configs / f"{mode}.yml").write_text(json.dumps(config, indent=2) + "\n")
