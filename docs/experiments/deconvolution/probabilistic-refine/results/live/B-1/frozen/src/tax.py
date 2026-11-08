"""Regional tax table (live-run task B)."""


def tax_rate(region):
    rates = {"eu": 0.20, "us": 0.07, "row": 0.00}
    return rates[region]


def with_tax(amount, region):
    return round(amount * (1 + tax_rate(region)), 2)
