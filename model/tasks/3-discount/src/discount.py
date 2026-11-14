def discount(customer, amount):
    if customer["tier"] == "gold":
        if amount > 100:
            rate = 0.2
        else:
            rate = 0.1
    elif customer["tier"] == "silver":
        if amount > 100:
            rate = 0.1
        else:
            rate = 0.05
    else:
        rate = 0
    result = amount * rate
    return result
