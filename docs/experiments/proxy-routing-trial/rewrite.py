"""Apply P02 only to the frozen experimental copy."""
from pathlib import Path

path = Path('.runs/proxy-routing-trial/after/mvibe_auth/http.py')
source = path.read_text(encoding="utf-8")
source = source.replace('''    for rule in no_proxy:
        if _no_proxy_rule_matches(rule.lower(), host, url.scheme, port):
            return True
    return False''', '''    return any(_no_proxy_rule_matches(rule.lower(), host, url.scheme, port) for rule in no_proxy)''')
source = source.replace('_ip_network_rule_matches(rule, host)', '_ip_rule_matches(rule, host, network=True)')
source = source.replace('_ip_literal_rule_matches(rule, host)', '_ip_rule_matches(rule, host, network=False)')
start = source.index('def _ip_network_rule_matches(')
end = source.index('def _host_rule_matches(', start)
source = source[:start] + '''def _ip_rule_matches(rule: str, host: str, *, network: bool) -> bool | None:
    try:
        target = ipaddress.ip_network(rule.strip("[]"), strict=False) if network else ipaddress.ip_address(rule)
    except ValueError:
        return None
    try:
        address = ipaddress.ip_address(host)
    except ValueError:
        return False
    return address in target if network else address == target


''' + source[end:]
start = source.index('    if "://" in rule:', source.index('def _parse_no_proxy_host_rule'))
end = source.index('    elif rule.count(":") == 1:', start)
source = source[:start] + '''    qualified = "://" in rule
    if qualified or rule.startswith("["):
        parsed = urlparse(rule if qualified else f"//{rule}")
        if qualified and parsed.scheme and parsed.scheme != scheme:
            return None
        try:
            port = parsed.port
        except ValueError:
            return None
        rule = parsed.hostname or ("" if qualified else rule)
''' + source[end:]
path.write_text(source, newline='\n')
