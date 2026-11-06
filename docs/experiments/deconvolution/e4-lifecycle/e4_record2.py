"""Held-out corpus shim (plan 6.6): `e4_record2.py record --out PATH` records with Hypothesis seed 20261107 and 48 generated
sequences through e4_oracle.record unchanged (same alphabet, normalisations, roots); `e4_record2.py replay --golden PATH
--l2 PATH` calls e4_oracle.replay_engine unchanged and writes the Level 2 file (first Level 1 divergence or none, pairs)."""
import json, sys, time
from pathlib import Path
import e4_oracle as O
O.SEED, O.N_GENERATED = 20261107, 48  # the only change against e4_oracle: a seed that did not exist before, twice the generated

def main(argv: list[str]) -> int:
    mode, opts = argv[0], dict(zip(argv[1::2], argv[2::2]))
    with O.embedded_engine() as engine:
        if mode == "record":
            print(json.dumps(O.record(engine, Path(opts["--out"]), None, O.N_GENERATED), indent=1)); return 0  # noqa: E702
        l1, l2 = O.replay_engine(engine, Path(opts["--golden"]))
    Path(opts["--l2"]).write_text(json.dumps({"written": time.time(), "root": str(O.ROOT), "golden": opts["--golden"], "level1": l1, "level2_ops": len(l2), "divergences": l2}, indent=1), encoding="utf-8")
    print(f"replay: level1 {l1 or 'no divergence'}; level2 differences {len(l2)} {l2[:5]}")
    return 0 if l1 is None and not l2 else 1

if __name__ == "__main__": sys.exit(main(sys.argv[1:]))  # noqa: E701
