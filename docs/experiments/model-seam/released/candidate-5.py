def total_price(items):
    return sum(item["price"] * item["count"] for item in items)
