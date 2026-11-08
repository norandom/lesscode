"""Cart pricing helpers (live-run task A)."""


def _round_2dp(value):
    """Round a value to 2 decimal places."""
    return round(value, 2)


def unit_price(item):
    table = {"apple": 1.25, "pear": 1.10, "fig": 2.40}
    return table[item]


def line_total(item, qty):
    return _round_2dp(unit_price(item) * qty)


def bulk_discount(qty, total):
    if qty >= 10:
        return _round_2dp(total * 0.90)
    return total


def cart_total(lines):
    raw = sum(line_total(item, qty) for item, qty in lines)
    return bulk_discount(sum(qty for _, qty in lines), raw)


def format_receipt(lines):
    parts = [f"{item} x{qty}: {line_total(item, qty):.2f}" for item, qty in lines]
    parts.append(f"total: {cart_total(lines):.2f}")
    return "; ".join(parts)


def save_receipt(conn, text):
    conn.write(("receipts", text))
