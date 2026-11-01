"""Source-bound prototype of direct row construction and named field selection."""
import ast
from pathlib import Path
import json

path = Path('.runs/facdrone-csv-trial/after/packages/facdrone-reporting/facdrone_reporting/library/normalized_csv.py')
source = path.read_text(encoding="utf-8")
tree = ast.parse(source)
lines = source.splitlines(True)
changes = []
families = []
for parent in ast.walk(tree):
    body = getattr(parent, 'body', None)
    if not isinstance(body, list):
        continue
    for index, statement in enumerate(body):
        if not (isinstance(statement, ast.Assign) and len(statement.targets) == 1
                and isinstance(statement.targets[0], ast.Name) and statement.targets[0].id == 'values'
                and isinstance(statement.value, ast.Call) and isinstance(statement.value.func, ast.Name)
                and statement.value.func.id == '_base_values'):
            continue
        updates = []
        position = index + 1
        while position < len(body) and isinstance(body[position], ast.Expr):
            call = body[position].value
            if not (isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute)
                    and isinstance(call.func.value, ast.Name) and call.func.value.id == 'values'
                    and call.func.attr == 'update' and len(call.args) == 1 and not call.keywords):
                break
            updates.append(call.args[0])
            position += 1
        if not updates or position == len(body) or ast.unparse(body[position]) != 'records.append(values)':
            continue
        indent = ' ' * statement.col_offset
        # Keep original field dictionaries, comments and per-field spelling.
        operands = [ast.get_source_segment(source, node) for node in [statement.value, *updates]]
        replacement = indent + 'records.append(\n' + indent + '    ' + '\n'.join(
            ([operands[0]] + [indent + '    | ' + operand for operand in operands[1:]])) + '\n' + indent + ')\n'
        changes.append((statement.lineno - 1, body[position].end_lineno, replacement))
        families.append({'line': statement.lineno, 'updates': len(updates)})
for start, end, replacement in sorted(changes, reverse=True):
    lines[start:end] = [replacement]
source = ''.join(lines)

# Consolidate only consecutive same-object, same-name canonical fields.
tree = ast.parse(source)
lines = source.splitlines(True)
changes, groups = [], []
for node in ast.walk(tree):
    if not isinstance(node, ast.Dict):
        continue
    group = []
    def flush():
        if len(group) >= 3:
            first, last = group[0], group[-1]
            names = ', '.join(repr(item[0].value) for item in group)
            indent = ' ' * first[0].col_offset
            text = f'{indent}**_fields({ast.unparse(first[1].args[0])}, {names}),\n'
            changes.append((first[0].lineno - 1, last[1].end_lineno, text))
            groups.append([item[0].value for item in group])
        group.clear()
    for key, value in zip(node.keys, node.values):
        eligible = (isinstance(key, ast.Constant) and isinstance(key.value, str)
            and isinstance(value, ast.Call) and isinstance(value.func, ast.Name) and value.func.id == '_field'
            and len(value.args) == 2 and isinstance(value.args[0], ast.Name)
            and isinstance(value.args[1], ast.Constant) and value.args[1].value == key.value)
        if not eligible or (group and ast.unparse(value.args[0]) != ast.unparse(group[0][1].args[0])):
            flush()
        if eligible:
            group.append((key, value))
    flush()
for start, end, replacement in sorted(changes, reverse=True):
    lines[start:end] = [replacement]
source = ''.join(lines)
if groups:
    source = source.replace('def _field(value:', '''def _fields(value: object, *names: str) -> dict[str, str]:
    """Select named canonical fields in declaration order, retaining full validation."""
    return {name: _field(value, name) for name in names}


def _field(value:''')
ast.parse(source)
path.write_text(source, newline='\n')
Path('docs/experiments/facdrone-csv-trial/rewrite-selection.json').write_text(
    json.dumps({'direct_row_construction': families, 'named_field_groups': groups}, indent=2) + '\n')
print(f'{len(families)} row constructors; {len(groups)} field groups')
