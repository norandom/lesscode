"""Show whole-module reduction without hiding the rejected comparison."""
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

root = Path(__file__).parent
fig, axes = plt.subplots(1, 2, figsize=(9, 3.5))
for ax, folder, title, verdict in zip(axes, [root, root.parent/'facdrone-csv-trial'],
        ['mvibe HTTP module', 'Facdrone normalized CSV'],
        ['1,172 tests pass on both copies', 'Rejected: marginal; publication gate fails on both']):
    result = json.loads((folder/'result.json').read_text(encoding="utf-8"))
    values = [result[side]['module']['physical_lines'] for side in ('before', 'after')]
    ax.barh(['Original', 'Candidate'], values, color=['#718096', '#285D8C'])
    for position, value in enumerate(values):
        ax.text(value + max(values)*.025, position, f'{value:,}', va='center')
    ax.set_xlim(0, max(values)*1.23)
    ax.invert_yaxis()
    ax.set_title(f'{title}\n{values[0]-values[1]} lines removed ({(1-values[1]/values[0]):.1%})', fontsize=11)
    ax.set_xlabel('Whole-module physical lines')
    ax.spines[['top','right']].set_visible(False)
    ax.text(0, -.48, verdict, transform=ax.transAxes, fontsize=8, wrap=True)
fig.subplots_adjust(left=.1, right=.98, top=.78, bottom=.35, wspace=.42)
fig.suptitle('Two isolated replacements: useful modest saving versus marginal change', fontsize=12)
matplotlib.rcParams.update({'svg.fonttype':'none','svg.hashsalt':'lesscode-paired-20261101'})
fig.savefig(root/'comparison.svg', bbox_inches='tight', metadata={'Date':None})
svg = root/'comparison.svg'
svg.write_text('\n'.join(line.rstrip() for line in svg.read_text(encoding="utf-8").splitlines())+'\n')
fig.savefig(root/'comparison.png', dpi=160, bbox_inches='tight')
