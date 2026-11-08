from cart import cart_total, format_receipt
from tax import with_tax

print(format_receipt([('apple', 2)], 'eu'), with_tax(1.0, 'us'))
