"""Independent, hand-derived NS01 outcomes; never learns answers from either side."""
import inspect
import json
import math
import os
from pathlib import Path
import unittest

from lesscode_baseline import contracts as base
from lesscode_baseline.rules import contracts as policy


class ExpectedOutcomes(unittest.TestCase):
    def invalid(self, call, message):
        with self.assertRaises(base.ContractError) as caught:
            call()
        self.assertIs(type(caught.exception), base.ContractError)
        self.assertEqual(str(caught.exception), message)

    def test_frozen_json_obligations(self):
        corpus = json.loads(Path(__file__).with_name('expected-outcomes.json').read_text())
        for case in corpus['cases']:
            with self.subTest(case=case):
                name = case['operation']
                kwargs = {}
                if name == '_object':
                    kwargs['fields'] = case['fields']
                if name == '_choice':
                    kwargs['choices'] = tuple(case['choices'])
                if name == '_relative':
                    kwargs['root'] = case.get('root', False)
                if name == '_sequence':
                    kwargs.update(reader=policy._string, identity=lambda value: value)
                call = lambda: getattr(policy, name)(case['input'], path='$case', **kwargs)
                if 'error' in case:
                    self.invalid(call, case['error'])
                elif 'expected_tuple' in case:
                    self.assertEqual(call(), tuple(case['expected_tuple']))
                    self.assertIs(type(call()), tuple)
                else:
                    self.assertEqual(call(), case['expected'])
                    self.assertIs(type(call()), type(case['expected']))

    def test_keywords_signatures_annotations(self):
        self.invalid(lambda: policy._fail(path='$case', reason='stop'), '$case: stop')
        expected = {
            '_fail': '(path: str, reason: str)',
            '_object': '(value, path: str, fields: str) -> dict',
            '_string': '(value, path: str) -> str',
            '_choice': '(value, path: str, choices: tuple[str, ...]) -> str',
            '_hash': '(value, path: str) -> str',
            '_sequence': '(value, reader, path: str, identity=None) -> tuple',
        }
        for name, signature in expected.items():
            self.assertEqual(str(inspect.signature(getattr(policy, name))), signature)

    def test_object_precedence_identity_order(self):
        self.invalid(lambda: policy._object({'z': 1, 'q': 2}, '$case', 'b a'), '$case.z: unknown field')
        self.invalid(lambda: policy._object({}, '$case', 'b a'), '$case.b: required field missing')
        value = {'b': 2, 'a': 1}
        self.assertIs(policy._object(value, '$case', 'a b'), value)
        self.assertEqual(list(value), ['b', 'a'])

    def test_sequence_domains_and_reader_order(self):
        for value in ('ab', {'a'}, {'a': 1}, None, 1):
            self.invalid(lambda: policy._sequence(value, policy._string, '$case'), '$case: expected array')
        self.assertEqual(policy._sequence(('a', 'b'), policy._string, '$case'), ('a', 'b'))
        calls = []
        def reader(value, path):
            calls.append((value, path))
            return policy._string(value, path)
        self.invalid(lambda: policy._sequence(['x', 'x', 'y'], reader, '$nested.items', lambda x: x),
                     "$nested.items[1]: duplicate identity 'x'")
        self.assertEqual(calls, [('x', '$nested.items[0]'), ('x', '$nested.items[1]'), ('y', '$nested.items[2]')])
        self.assertEqual(policy._sequence(['x', 'x'], policy._string, '$case'), ('x', 'x'))

    def test_numeric_path_time_boundaries(self):
        for value in (True, False, -1, float('inf'), -float('inf'), float('nan'), 10**1000, '1', None):
            self.invalid(lambda: policy._number(value, '$case'), '$case: expected finite nonnegative number')
        for value in (0, 1, -0.0, 1.5):
            self.assertIs(policy._number(value, '$case'), value)
        for value in ('/a', 'a//b', 'a/../b', 'a\\b', 'a:b', 'a ', 'a.'):
            self.invalid(lambda: policy._relative(value, '$case'), '$case: expected relative POSIX path without traversal or normalized components')
        for value in ('a/b', 'a*'):
            self.assertEqual(policy._relative(value, '$case'), value)
        for value in ('2026-11-01T00:00:00Z', '2026-11-01T00:00:00+00:00'):
            self.assertEqual(policy._utc(value, '$case'), value)
        self.invalid(lambda: policy._utc('bad', '$case'), '$case: expected UTC ISO-8601 timestamp')
        self.invalid(lambda: policy._utc('2026-11-01T00:00:00', '$case'), '$case: expected explicit UTC timezone')

    def test_public_records_serialization_and_nested_errors(self):
        value = {'root': '.', 'include': ['src/*.py'], 'exclude': []}
        record = policy.PolicyScope.from_dict(value)
        self.assertIs(type(record), policy.PolicyScope)
        self.assertEqual(record, policy.PolicyScope('.', ('src/*.py',), ()))
        self.assertEqual(record.to_dict(), value)
        self.assertEqual(json.loads(base.dumps(record)), value)
        self.invalid(lambda: policy.PolicyScope.from_dict({'root': '.', 'include': ['x', 'x'], 'exclude': []}),
                     "$input.include[1]: duplicate identity 'x'")
        self.invalid(lambda: policy.ChangeBudget.from_dict({'kind': 'absolute_increase', 'maximum': True}),
                     '$input.maximum: expected finite nonnegative number')
        self.assertEqual(policy.ChangeBudget.from_dict({'kind': 'absolute_increase', 'maximum': 0}),
                         policy.ChangeBudget('absolute_increase', 0))

    def test_adverse_controls(self):
        self.invalid(lambda: policy._sequence(['same', 'same'], policy._string, '$case', lambda x: x),
                     "$case[1]: duplicate identity 'same'")
        self.invalid(lambda: policy._hash('g' * 64, '$case'), '$case: expected lowercase SHA-256 fingerprint')
        self.invalid(lambda: policy._object({'extra': 1}, '$case', 'required'), '$case.extra: unknown field')


if __name__ == '__main__':
    print('IMPORTED_SOURCE=' + str(Path(policy.__file__).resolve()), flush=True)
    unittest.main(verbosity=2)
