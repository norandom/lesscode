"""Finite differential oracle over complete proxy decisions and transport effects."""
import argparse
import asyncio
import hashlib
import importlib.util
import itertools
import json
from pathlib import Path
import random
import sys

parser = argparse.ArgumentParser()
parser.add_argument('before', type=Path)
parser.add_argument('after', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
sys.path.insert(0, str(args.before))

def load(root, name):
    spec = importlib.util.spec_from_file_location(name, root / 'mvibe_auth/http.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

before, after = load(args.before, 'original_http'), load(args.after, 'candidate_http')

def observation(function, *values):
    try:
        return ['return', function(*values)]
    except Exception as error:
        return ['raise', type(error).__name__, str(error)]

rules = ['', '*', 'example.com', '.example.com', 'EXAMPLE.COM', 'localhost',
         '10.0.0.1', '10.0.0.0/8', '10.0.0.1/32', '10.0.0.1/33', '01.0.0.1',
         '::1', '[::1]', '2001:db8::/32', 'fe80::1%eth0', '[fe80::1%eth0]',
         '[::1]:443', '[::1]:bad', '[bad]', '[::1', 'example.com:443',
         'example.com:0', 'example.com:65536', 'example.com:bad', 'example.com:-1',
         'example.com:²', 'example.com:４４３', 'https://example.com',
         'http://example.com:80/path', 'https://[::1]:443', 'https://[fe80::1%eth0]',
         'https://', 'https://user:pass@example.com:443/path', 'https://a:99999',
         'a..b', '.','例子.测试', ' a ', 'a\nb', 'https://a／b', '//example.com']
hosts = ['example.com', 'sub.example.com', 'notexample.com', '.example.com', '',
         'localhost', '10.0.0.1', '10.2.3.4', '11.0.0.1', '::1', '::2',
         '2001:db8::1', 'fe80::1%eth0', 'fe80::1%eth1', 'fe80::1', 'a',
         '例子.测试', 'sub.例子.测试', '01.0.0.1']
randomizer = random.Random(20261101)
rules += [''.join(randomizer.choices('abc012.:/[]% *', k=randomizer.randrange(1, 28))) for _ in range(200)]
cases = itertools.product(rules, hosts, ['http', 'https', 'ftp', ''], [None, 0, 80, 443, 8080, 65536])
mismatches, counts = [], {'rule': 0, 'bypass': 0, 'transport': 0}
digest = hashlib.sha256()

def compare(kind, case, original, candidate):
    counts[kind] += 1
    digest.update(json.dumps([kind, case, original], ensure_ascii=True).encode())
    if original != candidate:
        mismatches.append({'kind': kind, 'input': case, 'before': original, 'after': candidate})

for case in cases:
    compare('rule', case, observation(before._no_proxy_rule_matches, *case),
            observation(after._no_proxy_rule_matches, *case))

urls = ['https://example.com', 'https://sub.example.com', 'http://10.2.3.4',
        'https://[::1]', 'https://[fe80::1%eth0]', 'https://example.com:8080',
        'ftp://example.com', '/relative', 'http://example.com:0']
for url, rule in itertools.product(urls, rules):
    for sequence in [(rule,), ('never.example', rule), (rule, '*'), ('*', rule)]:
        request_url = before.httpx2.URL(url)
        compare('bypass', [url, sequence], observation(before._should_bypass_proxy, request_url, sequence),
                observation(after._should_bypass_proxy, request_url, sequence))

async def transport(module, proxies, bypass):
    events = []
    class FakeTransport(module.httpx2.AsyncBaseTransport):
        def __init__(self, **kwargs):
            self.name = kwargs.get('proxy', 'direct')
            events.append(['create', kwargs])
        async def handle_async_request(self, request):
            events.append(['request', self.name, str(request.url)])
            return module.httpx2.Response(200)
        async def aclose(self):
            events.append(['close', self.name])
    original = module.httpx2.AsyncHTTPTransport
    module.httpx2.AsyncHTTPTransport = FakeTransport
    try:
        instance = module._EnvProxyTransport(proxies, bypass, trust_env=False)
        for url in urls[:-2]:
            await instance.handle_async_request(module.httpx2.Request('GET', url))
        await instance.aclose()
    finally:
        module.httpx2.AsyncHTTPTransport = original
    return events

for proxies, bypass in itertools.product([{}, {'https': 'https-proxy'}, {'all': 'all-proxy'},
        {'http': 'http-proxy', 'https': 'https-proxy', 'all': 'all-proxy'}],
        [(), ('*',), ('.example.com', '10.0.0.0/8', '::1'), ('example.com:8080',)]):
    compare('transport', [proxies, bypass], asyncio.run(transport(before, proxies, bypass)),
            asyncio.run(transport(after, proxies, bypass)))

# A relevant bad compression: treating .example.com as also matching the bare host.
assert before._no_proxy_rule_matches('.example.com', 'example.com', 'https', 443) is False
mutation_killed = before._no_proxy_rule_matches('example.com', 'example.com', 'https', 443) is True
result = {'counts': counts, 'mismatch_count': len(mismatches), 'mismatches': mismatches[:50],
          'reference_observations_sha256': digest.hexdigest(), 'leading_dot_mutation_killed': mutation_killed,
          'seed': 20261101, 'python': sys.version,
          'source_hashes': {name: hashlib.sha256((root / 'mvibe_auth/http.py').read_bytes()).hexdigest()
                            for name, root in [('before', args.before), ('after', args.after)]}}
args.output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({key: result[key] for key in ('counts', 'mismatch_count', 'leading_dot_mutation_killed')}))
sys.exit(bool(mismatches))
