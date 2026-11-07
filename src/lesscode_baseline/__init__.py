"""Local baseline evaluation package."""

__version__ = "0.1.0"

_LAZY = ("measures", "regenerator", "surface", "testinfra")
__all__ = list(_LAZY)


def __getattr__(name: str):
    if name in _LAZY:
        import importlib

        module = importlib.import_module(f".{name}", __name__)
        globals()[name] = module
        return module
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

