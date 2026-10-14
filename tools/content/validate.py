#!/usr/bin/env python3
"""Moves catalog items into Validate and back (sign-off), by id, editing the content JSON as text.

Every edit keeps the files' own formatting (only the values it changes, and the lines it adds or
removes, differ), and a file is written only once its new text is built and parses (and says what
was meant). Several sessions edit these files: run it on a clean file, or commit what it touched
right after (`git add -p`).

    python3 -B tools/content/validate.py list
        Validate's groups, their items, and where each goes back to (the plan's "Back to").
    python3 -B tools/content/validate.py in <id>... --group <group> [--label "Label"]
        Into Validate › <group> (made if missing; "outside" when the items' home group is).
        Prints each item's home, for the task table's "Back to" column.
    python3 -B tools/content/validate.py out <id>... [--to <category>/<group>]
    python3 -B tools/content/validate.py out --group <group> [--to <category>/<group>]
        Sign-off: back home. Without --to, each item goes back to its "Back to" in the plan's task
        tables (plan/*.md: `living / seating`; `living / —` is the category with no group).
        Validate groups left empty are dropped; the last one leaving is reported (16-sign-off.md
        says what to do then).

    Options: --dry-run (print the diff, write nothing), --content <dir> (default
    web/public/content: base.json and its includes), --plan <dir> (default plan/).

Free turning: whether a 1x1 item may turn freely comes from its category (`objectRules.freeRotation`
in base.json) unless the item says `"freeRotation": true|false`. Going into Validate, an item that
turns freely at home gets `"freeRotation": true` when Validate itself doesn't turn freely, so it keeps
turning while it's checked; signing off carries an item's own `"freeRotation"` back with it unchanged.
"""

import argparse
import difflib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


# --- A JSON parser that keeps where every value is in the text --------------------------------

class Node:
    """A JSON value and its span in the text; objects keep their members' key spans."""

    def __init__(self, kind, start, end, value=None, members=None, items=None):
        self.kind, self.start, self.end, self.value = kind, start, end, value
        self.members = members  # [(key, key_start, Node)] for objects
        self.items = items  # [Node] for arrays

    def get(self, key):
        for k, _, v in self.members or []:
            if k == key:
                return v
        return None

    def member(self, key):
        for i, (k, ks, v) in enumerate(self.members or []):
            if k == key:
                return i, ks, v
        return None


_WS = re.compile(r'\s*')
_NUM = re.compile(r'-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?')
_STR = re.compile(r'"(?:[^"\\]|\\.)*"', re.S)


def parse(text):
    def ws(i):
        return _WS.match(text, i).end()

    def value(i):
        i = ws(i)
        c = text[i]
        if c == '{':
            members, j = [], ws(i + 1)
            if text[j] == '}':
                return Node('object', i, j + 1, members=members)
            while True:
                j = ws(j)
                m = _STR.match(text, j)
                key = json.loads(m.group())
                j = ws(m.end())
                assert text[j] == ':', f'":" expected at {j}'
                v = value(j + 1)
                members.append((key, m.start(), v))
                j = ws(v.end)
                if text[j] == ',':
                    j += 1
                    continue
                assert text[j] == '}', f'"}}" expected at {j}'
                return Node('object', i, j + 1, members=members)
        if c == '[':
            items, j = [], ws(i + 1)
            if text[j] == ']':
                return Node('array', i, j + 1, items=items)
            while True:
                v = value(j)
                items.append(v)
                j = ws(v.end)
                if text[j] == ',':
                    j += 1
                    continue
                assert text[j] == ']', f'"]" expected at {j}'
                return Node('array', i, j + 1, items=items)
        if c == '"':
            m = _STR.match(text, i)
            return Node('string', i, m.end(), json.loads(m.group()))
        for word, v in (('true', True), ('false', False), ('null', None)):
            if text.startswith(word, i):
                return Node('literal', i, i + len(word), v)
        m = _NUM.match(text, i)
        assert m, f'a value expected at {i}'
        return Node('number', i, m.end(), json.loads(m.group()))

    return value(0)


def indent_at(text, pos):
    line = text.rfind('\n', 0, pos) + 1
    return text[line:pos] if text[line:pos].strip() == '' else ''


# --- The content files ------------------------------------------------------------------------

class File:
    def __init__(self, path):
        self.path = path
        self.text = path.read_text()
        self.tree = parse(self.text)
        self.edits = []  # (start, end, replacement)

    def objects(self):
        arr = self.tree.get('objects')
        return arr.items if arr else []

    def categories(self):
        arr = self.tree.get('buyCategories')
        return arr.items if arr else []

    def replace(self, node, text):
        self.edits.append((node.start, node.end, text))

    def set_member(self, obj, key, value, after=None):
        """Sets `key` to `value` (JSON text), adding it after member `after` (or last) if missing."""
        found = obj.member(key)
        if found:
            self.edits.append((found[2].start, found[2].end, value))
            return
        anchor = obj.member(after) if after else None
        _, ks, v = anchor or (None, *obj.members[-1][1:])
        sep = f',\n{indent_at(self.text, ks)}' if '\n' in self.text[obj.start:obj.end] else ', '
        self.edits.append((v.end, v.end, f'{sep}"{key}": {value}'))

    def remove_member(self, obj, key):
        found = obj.member(key)
        if not found:
            return
        i, ks, v = found
        if i > 0:  # from the end of the member before, through this one
            self.edits.append((obj.members[i - 1][2].end, v.end, ''))
        elif len(obj.members) > 1:  # the first: through to the next one's key
            self.edits.append((ks, obj.members[1][1], ''))
        else:
            self.edits.append((obj.start + 1, obj.end - 1, ''))

    def append_item(self, arr, text_one_line, text_lines):
        """Adds an element after the last, in the style of the ones there."""
        last = arr.items[-1]
        multi = '\n' in self.text[last.start:last.end]
        ind = indent_at(self.text, last.start)
        body = text_lines(ind) if multi else text_one_line
        sep = f',\n{ind}' if '\n' in self.text[arr.start:arr.end] else ', '
        self.edits.append((last.end, last.end, f'{sep}{body}'))

    def remove_item(self, arr, node):
        i = next(k for k, n in enumerate(arr.items) if n is node)
        if i > 0:
            self.edits.append((arr.items[i - 1].end, node.end, ''))
        elif len(arr.items) > 1:
            self.edits.append((node.start, arr.items[1].start, ''))
        else:
            self.edits.append((arr.start + 1, arr.end - 1, ''))

    def new_text(self):
        text = self.text
        last = None
        for start, end, rep in sorted(self.edits, key=lambda e: (e[0], e[1]), reverse=True):
            assert last is None or end <= last, f'{self.path}: overlapping edits'
            text = text[:start] + rep + text[end:]
            last = start
        return text


def load(content):
    base = File(content / 'base.json')
    includes = json.loads(base.text).get('include', [])
    return [base] + [File(content / name) for name in includes]


def back_to_table(plan):
    """Item id -> (category, group or None) from the task tables' "Back to" column."""
    homes = {}
    for md in sorted(plan.glob('*.md')):
        column = None
        for line in md.read_text().splitlines():
            # A list instead (16-sign-off.md's sinks): "- `a`, `b`: bathroom / sinks;".
            listed = re.match(r'- ((?:`[^`]+`(?:, )?)+): ([\w ]+) / ([^;.]+)[;.]?$', line)
            if listed:
                group = listed.group(3).strip()
                for ident in re.findall(r'`([^`]+)`', listed.group(1)):
                    homes.setdefault(ident, (listed.group(2).strip(), None if group in ('—', '-') else group))
            if not line.startswith('|'):
                column = None
                continue
            cells = [c.strip() for c in line.strip().strip('|').split('|')]
            if 'Back to' in cells:
                column = cells.index('Back to')
                continue
            if column is None or column >= len(cells):
                continue
            ident = re.search(r'`([^`]+)`', cells[0])
            home = cells[column].split('/')
            if ident and len(home) == 2:
                cat, group = home[0].strip(), home[1].strip()
                homes[ident.group(1)] = (cat, None if group in ('—', '-', '') else group)
    return homes


def label_of(group):
    return ' '.join(w[:1].upper() + w[1:] for w in re.split(r'[\s_-]+', group))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('what', choices=['list', 'in', 'out'])
    ap.add_argument('ids', nargs='*')
    ap.add_argument('--group')
    ap.add_argument('--label')
    ap.add_argument('--to')
    ap.add_argument('--dry-run', action='store_true')
    ap.add_argument('--content', type=Path, default=ROOT / 'web/public/content')
    ap.add_argument('--plan', type=Path, default=ROOT / 'plan')
    args = ap.parse_args()

    files = load(args.content)
    base = files[0]
    by_id = {}
    for f in files:
        for o in f.objects():
            ident = o.get('id')
            if ident is not None:
                by_id[ident.value] = (f, o)
    cats = {}
    for f in files:
        for c in f.categories():
            cats[c.get('id').value] = (f, c)
    if 'validate' not in cats:
        sys.exit('validate: there is no Validate category in the content')
    vfile, vcat = cats['validate']
    vgroups = vcat.get('groups')

    def where(o):
        cat = o.get('category')
        group = o.get('group')
        return (cat.value if cat else None, group.value if group else None)

    def groups_of(cat):
        arr = cats[cat][1].get('groups') if cat in cats else None
        return {g.get('id').value: g for g in (arr.items if arr else [])}

    homes = back_to_table(args.plan) if args.plan.is_dir() else {}
    rules = json.loads(base.text).get('objectRules', {})
    free_cats = set(rules.get('freeRotation', []))

    if args.what == 'list':
        in_validate = {}
        for ident, (f, o) in by_id.items():
            cat, group = where(o)
            if cat == 'validate':
                in_validate.setdefault(group, []).append(ident)
        for g in vgroups.items:
            gid = g.get('id').value
            items = in_validate.pop(gid, [])
            print(f'{gid} ({len(items)})')
            for ident in items:
                home = homes.get(ident)
                print(f'  {ident}  ->  {f"{home[0]} / {home[1] or chr(8212)}" if home else "(no Back to in plan/)"}')
        for gid, items in in_validate.items():
            print(f'{gid} (not a Validate group!): {", ".join(items)}')
        return

    ids = list(args.ids)
    if args.what == 'out' and args.group:
        ids += [i for i, (f, o) in by_id.items() if where(o) == ('validate', args.group) and i not in ids]
    if not ids:
        sys.exit('validate: which items? (ids, or --group with out)')
    missing = [i for i in ids if i not in by_id]
    if missing:
        sys.exit(f'validate: no such item: {", ".join(missing)}')

    if args.what == 'in':
        if not args.group:
            sys.exit('validate: in needs --group <validate group>')
        existing = groups_of('validate')
        outside = False
        for ident in ids:
            f, o = by_id[ident]
            cat, group = where(o)
            if cat == 'validate':
                sys.exit(f'validate: {ident} is in Validate already ({group})')
            if cat not in cats:
                sys.exit(f'validate: {ident} has no known category ({cat})')
            home_group = groups_of(cat).get(group)
            outside |= bool(home_group and home_group.get('outside') and home_group.get('outside').value)
            f.replace(o.get('category'), '"validate"')
            f.set_member(o, 'group', json.dumps(args.group), after='category')
            free = o.get('freeRotation')
            if free is None and cat in free_cats and 'validate' not in free_cats:
                f.set_member(o, 'freeRotation', 'true', after='category')
            print(f'{ident}: {cat} / {group or chr(8212)} -> validate / {args.group}')
        if args.group not in existing:
            label = json.dumps(args.label or label_of(args.group))
            gid = json.dumps(args.group)
            extra = ', "outside": true' if outside else ''
            vfile.append_item(
                vgroups,
                f'{{"id": {gid}, "label": {label}{extra}}}',
                lambda ind: f'{{\n{ind}  "id": {gid},\n{ind}  "label": {label}' + (f',\n{ind}  "outside": true' if outside else '') + f'\n{ind}}}',
            )
            print(f'new Validate group: {args.group}{" (outside)" if outside else ""}')
        expect = {i: ('validate', args.group) for i in ids}
    else:
        expect = {}
        emptied = set()
        for ident in ids:
            f, o = by_id[ident]
            cat, group = where(o)
            if cat != 'validate':
                sys.exit(f'validate: {ident} is not in Validate ({cat} / {group})')
            if args.to:
                to_cat, _, to_group = args.to.partition('/')
                home = (to_cat.strip(), to_group.strip() or None)
            elif ident in homes:
                home = homes[ident]
            else:
                sys.exit(f'validate: no "Back to" for {ident} in {args.plan}: give --to <category>/<group>')
            if home[0] not in cats:
                sys.exit(f'validate: {ident}: no category "{home[0]}"')
            if home[1] is not None and home[1] not in groups_of(home[0]):
                sys.exit(f'validate: {ident}: category "{home[0]}" has no group "{home[1]}" ({", ".join(groups_of(home[0]))})')
            f.replace(o.get('category'), json.dumps(home[0]))
            if home[1] is None:
                f.remove_member(o, 'group')
            else:
                f.set_member(o, 'group', json.dumps(home[1]), after='category')
            emptied.add(group)
            expect[ident] = home
            print(f'{ident}: validate / {group} -> {home[0]} / {home[1] or chr(8212)}')
        # Validate groups nothing is left in.
        still = {where(o)[1] for i, (f, o) in by_id.items() if i not in expect and where(o)[0] == 'validate'}
        dropped = [g for g in vgroups.items if g.get('id').value in emptied - still]
        for g in dropped:
            vfile.remove_item(vgroups, g)
            print(f'dropped the empty Validate group: {g.get("id").value}')
        if len(dropped) == len(vgroups.items):
            print('Validate is empty now: remove the category and its icon (plan/16-sign-off.md, "The end").')

    # Build, parse and check every changed file before writing any.
    out = []
    for f in files:
        if not f.edits:
            continue
        text = f.new_text()
        data = json.loads(text)
        for o in data.get('objects', []):
            if o.get('id') in expect:
                cat, group = expect[o['id']]
                assert (o.get('category'), o.get('group')) == (cat, group), f'{o["id"]}: {o.get("category")} / {o.get("group")}'
        out.append((f, text))
    for f, text in out:
        rel = f.path.relative_to(ROOT) if f.path.is_relative_to(ROOT) else f.path
        if args.dry_run:
            sys.stdout.writelines(difflib.unified_diff(f.text.splitlines(True), text.splitlines(True), f'a/{rel}' if not rel.is_absolute() else str(rel), f'b/{rel}' if not rel.is_absolute() else str(rel)))
        else:
            f.path.write_text(text)
            print(f'wrote {rel}')


if __name__ == '__main__':
    main()
