"""Regional tax table (live-run task B)."""


def tax_rate(region):
    rates = {"eu": 0.20, "us": 0.07, "row": 0.00}
    return rates[region]


def _apply_tax(amount, region):
    """Helper: multiply by (1 + tax_rate) and round to 2 decimals."""
    return round(amount * (1 + tax_rate(region)), 2)


def with_tax(amount, region):
    return _apply_tax(amount, region)
