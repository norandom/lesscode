def record_order(session, order):
    total = 0
    for line in order["lines"]:
        total = total + line["price"] * line["count"]
    order["total"] = total
    session.add(order)
    session.commit()


def record_refund(session, order, amount):
    order["total"] = order["total"] - amount
    order["refunded"] = True
    session.add(order)
    session.commit()
