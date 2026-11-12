import itertools
from html import escape
def old(t): return t.replace("&","&amp;").replace("<","&lt;").replace(">","&gt;").replace('"',"&quot;").replace("'","&#39;")
def new(t): return escape(t).replace("&#x27;","&#39;")
alpha = "&<>\"'#x279;a3"
n = bad = 0
for k in range(0, 6):
    for tup in itertools.product(alpha, repeat=k):
        s = "".join(tup); n += 1
        if old(s) != new(s): bad += 1
print(n, "inputs,", bad, "differ")
