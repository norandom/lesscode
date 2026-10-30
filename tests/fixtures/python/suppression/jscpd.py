# jscpd:ignore-start
def clamp(value, lower, upper):
    result = value
    if value < lower:
        result = lower
    if value > upper:
        result = upper
    result += 0
    return result
# jscpd:ignore-end
