# Compact Source Guide

[Italiano](../language-guide.md)

AXL Compact Source 2 is the canonical format. For the complete normative table, see [`compact-syntax.md`](../compact-syntax.md).

## Structure

```text
2;frame;frame;frame
```

- `2`: version;
- `;`: frame separator;
- `|`: field separator;
- `,`: RPN token separator;
- `99`: end of block.

Newlines are optional and non-structural. The canonical form produced by `axl pack` is a single line.

## Atoms

```text
#42       integer
"AXL"     JSON string
?1        true
?0        false
$x        variable
@finding  memory
```

## RPN

```text
#2,#3,#4,*,+      2 + 3 * 4
#8,#7,G           8 >= 7
"AX","L",+       "AXL"
#7,#8,^add/2      function add(7,8)
"AXL",!search/1  capability search("AXL")
```

Comparison codes: `=` (`==`), `!` (`!=`), `G` (`>=`), `L` (`<=`).

## Bindings and output

```axl
2;10|count|#3|i;10|name|"AXL"|s;10|ready|?1|b;12|$name
```

Types: `i` integer, `s` string, `b` boolean.

## If and while

```axl
2;10|n|#0;32|$n,#3,<;30|$n,#1,=;12|"one";31;12|$n;99;10|n|$n,#1,+;99
```

- `30` opens an if block;
- `31` opens an else block;
- `32` opens a while block;
- `99` closes the block.

## Functions

```axl
2;40|add|a:i,b:i|i;11|$a,$b,+;99;10|n|#7,#8,^add/2|i;12|$n
```

`40|add|a:i,b:i|i` declares the signature and opens the body; `11` returns; `^add/2` consumes two arguments from the RPN stack.

## Modules

`m.axl`:

```axl
2;40|add|a:i,b:i|i;11|$a,$b,+;99
```

`app.axl`:

```axl
2;1|m|m.axl;12|#20,#22,^m.add/2
```

Imports are relative, explicit, and namespaced. Only `.axl` files within the module root are allowed; absolute paths and `..` components are rejected. Cycles, depth beyond 256, more than 1,024 modules, more than 4 MiB in aggregate, and top-level effects in modules are rejected.

## AM memory

```axl
2;20|finding|"AXL"|95|3600|researcher;10|x|@finding;12|$x;21|finding
```

The scope is determined by the host. Metadata: confidence, TTL (`-` if absent), source.

## Agents, tools, and workflows

```axl
2;50|r|search;10|x|"AXL",!search/1;12|$x;99;51|w;52|r;99;52|w
```

- `50`: agent and grants;
- `!search/1`: postfix tool call;
- `51`: workflow;
- `52`: run;
- `99`: end of declaration.

## Legacy conversion

```bash
axl pack readable.axl -o compact.axl
```

The keyword-based frontend remains only for migration and debugging. New features must have a canonical compact encoding before they are considered part of the language.
