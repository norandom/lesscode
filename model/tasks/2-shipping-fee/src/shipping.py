FREE_FROM = 50


def _rate_for(weight):
    if weight <= 1:
        rate = 4
    elif weight <= 5:
        rate = 7
    else:
        rate = 12
    return rate


def shipping_fee(order):
    total = order["total"]
    weight = order["weight"]
    if total >= FREE_FROM:
        fee = 0
    else:
        fee = _rate_for(weight)
    return fee
