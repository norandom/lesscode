"""R39: expected-behavior assertions against the retained Python reference."""
import json
from lesscode_baseline import regenerator as rg, surface


def run_loop(tmp_path, sources, *, threshold=1.0, rollouts=1, runtime=True):
    frozen = tmp_path / "frozen"
    (frozen / "src").mkdir(parents=True)
    (frozen / "src/mod.py").write_text("def f():\n    return 1\n")
    (frozen / "src/user.py").write_text("from mod import f\n")
    report = surface.derive_surface(frozen, ["src/mod.py"], [], [])
    (frozen / "public-surface.json").write_text(json.dumps(report))
    req = rg.TurnRequest(str(frozen), ["src/mod.py"], "public-surface.json", "d", {}, "c", "fixture oracle")
    prompts = []
    source_iter = iter(sources)

    def model(system, user, params):
        prompts.append(user)
        return json.dumps({"files": [{"path": "src/mod.py", "content": next(source_iter)}],
                           "structure": {"pure": [], "shell": []}, "change_log": "probe"}), {"stop_reason": "end_turn"}

    def evaluator(root, results):
        if "return 1" in (root / "src/mod.py").read_text():
            return 0, None, {}
        return 1, "R39_known_counterexample", {}

    cand = tmp_path / "cand"
    outcome = rg.run_refine_loop(req, cand, frozen, tmp_path / "receipt.json", "reference",
        rg.RefineBudget(max_turns=len(sources) // rollouts, max_static_retries=1,
                       n_rollouts_per_turn=rollouts, threshold=threshold),
        runtime_evaluator=evaluator if runtime else None, call_fn=model)
    return outcome, prompts, (cand / "src/mod.py").read_text()


def test_runtime_counterexample_reaches_next_prompt(tmp_path):
    _, prompts, _ = run_loop(tmp_path, ["def f():\n    return 2\n", "def f():\n    return 1\n"])
    assert "R39_known_counterexample" in prompts[1]


def test_best_candidate_is_retained(tmp_path):
    outcome, _, source = run_loop(tmp_path, ["def f():\n    return 1\n", "def f():\n    return 2\n"], rollouts=2)
    assert outcome.status == "accepted"
    assert source == "def f():\n    return 1\n", outcome


def test_search_threshold_is_not_validity(tmp_path):
    outcome, _, _ = run_loop(tmp_path, ["def f():\n    return 2\n"], threshold=0.5, runtime=False)
    assert not outcome.validity, outcome


def test_absent_measurements_are_not_compression(tmp_path):
    outcome, _, _ = run_loop(tmp_path, ["def f():\n    return 1\n"])
    assert not outcome.compression_met, outcome


def test_still_failing_counterexample_remains_unresolved(tmp_path):
    outcome, _, _ = run_loop(tmp_path, ["def f():\n    return 2\n"])
    assert "R39_known_counterexample" in outcome.unresolved_obligations, outcome.unresolved_obligations
