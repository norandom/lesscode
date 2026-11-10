# Pack authoring guide, version 3

This is for mutation testing, the software testing technique that measures
how well a project's own test suite detects small deliberate changes to its
code. Each changed copy is run only against that project's tests in an
isolated checkout and is then discarded; nothing is deployed or executed
elsewhere. The tool that makes the changes is called the mutation engine.

The mutation engine reads one small JSON data pack per programming language.
The engine itself knows no language. It parses source files with the
tree-sitter grammar that ast-grep ships for the language, finds syntax nodes
the pack names, and rewrites them with the templates the pack gives. You
write the pack for one language. You are given packs for other languages as
examples of the format, not as content to copy: node kind names differ
between grammars.

## Fields

All node kind names must be exact node type names of the tree-sitter grammar
for the target language. A name that does not exist in the grammar makes the
whole pack invalid.

| Field | Meaning |
| --- | --- |
| `language` | The language name as ast-grep spells it, lower case |
| `comment_kinds` | Node kinds of comments. Nothing inside them is ever touched |
| `string_kinds` | Node kinds of string literals, including multi-part or template strings. Nothing inside them is ever touched |
| `unwrap_kinds` | Node kinds of a parenthesis wrapper that belongs to a statement's syntax, as in `if (x)`. When a condition is such a wrapper, the engine works on the expression inside it. Empty when conditions are not wrapped |
| `test_attribute_markers` | Text fragments of attributes or annotations that mark test-only code inside a production file. Empty when the language keeps tests in separate files |
| `test_attribute_kinds` | Node kinds of the attribute or annotation items that can stand directly in front of a declaration and carry one of the `test_attribute_markers`. Required when `test_attribute_markers` is not empty; leave it out otherwise |
| `kinds.branch_condition.field_of` | Pairs `[statement kind, field name]`: the named field of that statement kind holds a condition that decides control flow. List the conditional and loop statements and conditional expressions whose grammar rule has such a field |
| `kinds.branch_condition.returned` | `{"of", "kinds", "tokens"}`: `of` lists the node kinds of a return; `kinds` the node kinds of logical, comparison and negation expressions; `tokens` the operator tokens that make such an expression a yes-or-no value. A returned expression of one of these kinds with one of these operators is a condition |
| `kinds.failure_state.nodes` | Node kinds that signal failure to the caller: raising or throwing, returning, propagating an error |
| `kinds.failure_state.nodes_with_prefix` | Optional. Node kind mapped to text prefixes, for failure signals that are ordinary calls or macros in this language and can only be told apart by how they begin |
| `kinds.error_order.nodes` | The node kind of a conditional statement that can directly follow another one of the same kind. Always with `"requires_next_sibling": true` |
| `kinds.state_write.nodes` | Node kinds of assignments and in-place updates |
| `kinds.state_write.call_statements` | Node kinds of a call, and of an awaited call where the language has one, that can stand alone as a statement. Such a statement counts as a write made through a call |
| `kinds.pure_shell_boundary.nodes` | Node kinds of function and method definitions |
| `noop_statement` | Text of a statement that does nothing and is valid wherever a statement may stand |
| `statement_wrappers` | Node kinds that wrap an expression to make it a statement of its own |
| `statement_terminator` | The token that ends a simple statement, if the language has one; leave it out otherwise |

## Operators

`operators` tells the engine how to rewrite a site. Templates may use
`{text}` (the node's text), `{noop}` (the `noop_statement`) and `{child0}`
(the text of the node's first named child).

| Entry | Meaning |
| --- | --- |
| `operators.branch_condition.template` | Replaces a condition by its logical negation. The result must be valid syntax in the place where the condition stood, for any condition expression |
| `operators.branch_condition.token_swaps` | Pairs of operator tokens that can replace each other without changing the type of the expression: equal and not equal, a strict and a non-strict comparison, logical and and or. Spell them exactly as the language does |
| `operators.branch_condition.negation_swaps` | The pairs among `token_swaps` whose swap gives exactly the logical negation of the comparison: equal and not equal, in every spelling the language has. The engine uses it to avoid producing the same change twice. Do not list a strict and a non-strict comparison, or and and or |
| `operators.branch_condition.exclude_kinds` | Optional. Node kinds that can appear in a condition position but cannot be negated by the template, such as a pattern-binding condition or an empty condition |
| `operators.failure_state.by_kind` | Node kind mapped to a replacement template that removes the failure. Use `{noop}` for a statement that can simply be dropped. Leave out kinds for which no replacement is valid in general, such as returning a value |
| `operators.failure_state.returned_literal_swaps` | Pairs of literals that can replace each other as a returned value: the two boolean literals, and 0 and 1 |
| `operators.state_write` | Always `{"statement": "{noop}"}`: a write that is a statement of its own is replaced by the empty statement |
| `operators.error_order` | Always `{"swap_next": true}` |

## Rules

- Be precise rather than broad: a wrong kind produces changes that do not
  parse or do not compile.
- Do not include anything specific to a particular project, library or file.
- Keep the pack small: at most 40 lines and 2,500 characters, formatted like
  the examples.

## Output

Return only the JSON object of the pack, with no text before or after it.
