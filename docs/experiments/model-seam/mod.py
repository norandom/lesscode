def total_price(items):
    total = 0
    for item in items:
        if item["kind"] == "book":
            total = total + item["price"] * item["count"]
        elif item["kind"] == "food":
            total = total + item["price"] * item["count"]
        else:
            total = total + item["price"] * item["count"]
    return total
