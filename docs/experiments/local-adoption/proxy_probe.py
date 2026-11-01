"""Finite replacement-feasibility probe; never imports or rewrites mvibe."""

import argparse
import ast
import hashlib
import ipaddress
import json
from pathlib import Path
from urllib.parse import urlparse
from urllib.request import proxy_bypass_environment


def run(source, expected_hash):
    raw = source.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected_hash:
        raise ValueError("Source changed; re-admit the probe before execution")
    names = {"_no_proxy_rule_matches", "_ip_network_rule_matches",
             "_ip_literal_rule_matches", "_host_rule_matches", "_parse_no_proxy_host_rule"}
    nodes = [n for n in ast.parse(raw).body if isinstance(n, ast.FunctionDef) and n.name in names]
    if len(nodes) != len(names) or any(n.decorator_list for n in nodes):
        raise ValueError("Reviewed helper extraction no longer matches")
    namespace = {"ipaddress": ipaddress, "urlparse": urlparse}
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(source), "exec"), namespace)
    cases = [
        ("wildcard", "*", "any.example", "https", 443, True),
        ("exact_host", "example.com", "example.com", "https", 443, True),
        ("subdomain", "example.com", "sub.example.com", "https", 443, True),
        ("leading_dot_bare_host", ".internal.example", "internal.example", "https", 443, False),
        ("leading_dot_subdomain", ".internal.example", "api.internal.example", "https", 443, True),
        ("ipv4_cidr", "10.0.0.0/8", "10.1.2.3", "https", 443, True),
        ("ipv6_cidr", "2001:db8::/32", "2001:db8::1", "https", 443, True),
        ("scheme_and_port", "https://api.example:443", "api.example", "https", 443, True),
        ("scheme_mismatch", "http://api.example:443", "api.example", "https", 443, False),
        ("port_mismatch", "example.com:80", "example.com", "https", 443, False),
        ("ip_literal", "127.0.0.1", "127.0.0.1", "http", 80, True),
    ]
    observations = []
    for name, rule, host, scheme, port, expected in cases:
        original = namespace["_no_proxy_rule_matches"](rule, host, scheme, port)
        if original != expected:
            raise ValueError(f"Baseline case assumption failed: {name}")
        authority = f"[{host}]" if ":" in host else host
        candidate = proxy_bypass_environment(f"{authority}:{port}", {"no": rule})
        observations.append({"case": name, "rule": rule, "host": host, "scheme": scheme,
                             "port": port, "original": original, "candidate": candidate,
                             "match": original == candidate})
    if hashlib.sha256(source.read_bytes()).hexdigest() != expected_hash:
        raise ValueError("Source changed during probe")
    mismatches = sum(not case["match"] for case in observations)
    return {"source_sha256": expected_hash, "cases": observations, "mismatches": mismatches,
            "disposition": "rejected_behavior" if mismatches else "finite_cases_match_only",
            "application_tests_run": False, "adopted_production_delta": 0}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    baseline = json.loads(args.baseline.read_text(encoding="utf-8-sig"))
    entry = next(f for f in baseline["files"] if f["path"] == "mvibe_auth/http.py")
    result = run(Path(baseline["root"]) / entry["path"], entry["sha256"])
    with args.out.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(f"{result['disposition']}: {result['mismatches']} / {len(result['cases'])} mismatches")
