# AXL Compact Source 2

[Italiano](../compact-syntax.md)

AXL Compact Source 2 is the language's compact syntax. It keeps token count low
and parsing deterministic, while source files are normally laid out over multiple lines.

## Form

```text
2;frame;frame;frame
```

- `2` is the source format version;
- `;` separates frames;
- `|` separates fields;
- `,` separates tokens in an RPN expression;
- `99` closes a block;
- spaces, indentation, and newlines have no structural meaning;
- strings use JSON escaping, so they may contain separators.

Formatted example:

```axl
2;
10|x|#2,#3,#4,*,+|i; 12|$x;
```

Semantically equivalent to: compute `2 + 3 * 4`, assign it to `x: int`, emit `x`.

## RPN Expressions

Expressions use Reverse Polish Notation. No parentheses or precedence rules are required.

| Token | Meaning |
|---|---|
| `#42` | integer |
| `"AXL"` | JSON string |
| `?1`, `?0` | true/false booleans |
| `$x` | local variable |
| `@k` | memory recall |
| `+ - * /` | arithmetic |
| `= ! > < G L` | `== != > < >= <=` |
| `^f/2` | call function `f` with 2 values from the stack |
| `!t/1` | call tool `t` with 1 value from the stack |
| `~3` | build a list with 3 values from the stack |
| `%2` | build a map with 2 key/value pairs from the stack |

```text
#2,#3,#4,*,+      → 2 + (3 * 4)
"AX","L",+       → "AXL"
#7,#8,^add/2      → add(7,8)
"AXL",!search/1  → tool search("AXL")
#1,#2,#3,~3       → list<int>[1,2,3]
"a",#1,"b",#2,%2 → map<string,int>{"a":1,"b":2}
```

## Types

| Code | Type |
|---|---|
| `i` | int |
| `s` | string |
| `b` | bool |
| `li` | list&lt;int&gt; |
| `ls` | list&lt;string&gt; |
| `lb` | list&lt;bool&gt; |
| `msi` | map&lt;string,int&gt; |
| `msli` | map&lt;string,list&lt;int&gt;&gt; |

Collection codes are prefixed and self-delimiting: `lT` denotes `list<T>`,
while `mKV` denotes `map<K,V>`. Maps are immutable and homogeneous in their keys
and values, and require unique scalar keys. The CLI always uses the unambiguous
canonical wrapper `{\"$ax.map\":[[key,value],...]}`. In this tranche, map support
covers Compact Source and the reference runtime; AX-IR and SQLite persistence
remain later milestones.

The `l` prefix is recursive up to 16 levels: `lli` represents `list<list<int>>`. Lists are immutable and homogeneous; `~0` creates an empty list whose element type is determined by the declared context.

## Opcodes

| Opcode | Frame | Semantics |
|---:|---|---|
| `1` | `1|alias|path` | top-level relative `.axl` import, confined to the module root |
| `10` | `10|name|expr[|type]` | local binding |
| `11` | `11|expr` | return |
| `12` | `12|expr` | emit |
| `20` | `20|key|expr[|confidence|ttl|source]` | memory write |
| `21` | `21|key` | forget |
| `30` | `30|condition` | open if |
| `31` | `31` | else |
| `32` | `32|condition` | open while |
| `40` | `40|name|a:i,b:i|i` | open typed function |
| `50` | `50|name[|tool,tool]` | open agent with grants |
| `51` | `51|name` | open workflow |
| `52` | `52|name` | run agent/workflow |
| `99` | `99` | close the current block |

### Memory with Metadata

```axl
2;20|finding|"AXL"|95|3600|researcher;12|@finding;21|finding
```

Use `-` when TTL is absent:

```axl
2;20|k|"v"|100|-|runtime
```

### Function

```axl
2;40|add|a:i,b:i|i;11|$a,$b,+;99;12|#20,#22,^add/2
```

### Control Flow

```axl
2;10|n|#0;32|$n,#3,<;30|$n,#1,=;12|"one";31;12|$n;99;10|n|$n,#1,+;99
```

### Agent and Workflow

```axl
2;50|r|search;10|x|"AXL",!search/1;12|$x;99;51|w;52|r;99;52|w
```

## Compact UI Source 3

Application UI uses a `.ui.axl` sidecar with header `3`. It keeps the same
separators and multiline rule while adding frames for the semantic UI tree.

| Opcode | Frame | Meaning |
|---:|---|---|
| `60` | `60|view_id` | opens a view |
| `61` | `61|node_id|component_id` | opens a component |
| `62` | `62|property_id|value` | assigns a property |
| `63` | `63|event_id|action` | binds an event |
| `99` | `99` | closes a component or view |

Components `64` and `65` define a data table and a column. The table contract
uses resource (`1`), entity label (`2`), page size (`3`), density (`4`), and
mobile mode (`5`). The column contract uses field (`1`), label (`2`), visual type
(`3`), responsive priority from 1 to 3 (`4`), and minimum width from 80 to 600
pixels (`5`).

```axl
3;60|2;
  61|1000|64;
    62|1|"customers";
    62|2|"Customers";
    62|3|#20;
    62|4|"comfortable";
    62|5|"cards";
    61|1001|65; 62|1|"name"; 62|2|"Name"; 62|3|"text"; 62|4|#1; 62|5|#180; 99;
    61|1002|65; 62|1|"status"; 62|2|"Status"; 62|3|"status"; 62|4|#2; 99;
  99;
99;
```

The compiler rejects unknown components, out-of-contract properties, invalid
types, and columns without fields. This keeps the format compact without losing
semantic validation.

## Canonicalization

The `pack` command converts legacy syntax to the canonical representation:

```bash
axl pack legacy.axl -o packed.axl
```

`pack` produces the standard multiline format at width 100. `axl fmt` formats an
existing file; `--width N` selects the width and `--check` validates formatting
without changing the file.

The compiler can separately derive a canonical minified representation for
hashing, caching, deduplication, signing, and transport. Minified source is not
the recommended format for files committed to Git.

## Legacy Format

The readable text parser (`let`, `fn`, `agent`, visual indentation) remains temporarily available as a migration and debugging frontend. It is not the primary syntax and does not define AXL's future design.

## Evolution

New opcodes can be added without making the language verbose. Future families will include collections, records, enums, tasks, events, concurrency, standard capabilities, and platform bindings. The initial version prevents ambiguous interpretations; incompatible changes require a new source header.
