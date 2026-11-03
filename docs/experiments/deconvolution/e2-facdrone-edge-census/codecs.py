"""E2 frozen codec list (plan step 3). Data only, shared by the recorder and the static pass.

Entries ending in ".*" mean every public callable of that module. PREFIX_METHODS match bound
methods of pydantic models by name prefix. NONLITERAL_CTORS count only when called with an
argument (dynamic) or with a non-literal argument (static).
"""

CODECS = (
    "json.dumps", "json.loads",
    "base64.*", "zlib.*", "gzip.*",
    "pickle.dump", "pickle.dumps", "pickle.load", "pickle.loads",
    "dataclasses.asdict",
)
PREFIX_METHODS = ("model_dump", "model_validate")
NONLITERAL_CTORS = ("str", "bytes")
