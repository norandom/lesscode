def total_price(items):
    total = 0
    for item in items:
        total = total + item["price"] * item["count"]
    return total
