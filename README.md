KSS is an interpreted scripting language with explicit "core" and "plugin"
imports, events, json objects, and a built-in arbitrary-precision number
type `ncti`. Program files are `.kss`.

install command
```bash
curl -fsSL https://raw.githubusercontent.com/uyruyr777/kli-s-s/main/don.sh | bash
```
- [`kss-system.md`](kss-system.md) — the `system` core (console, files, `rand`)
- [`kss-math.md`](kss-math.md) — the `math` core and the `ncti` type
- [`kss-render.md`](kss-render.md) — the `render` core (windows and drawing)

---

## Contents

1. [General Structure](#1-general-structure)
2. [Comments](#2-comments)
3. [Imports: Cores and Plugins](#3-imports-cores-and-plugins)
4. [Data Types](#4-data-types)
5. [Variables](#5-variables)
6. [Reserved Words](#6-reserved-words)
7. [Assignment Sugar](#7-assignment-sugar)
8. [Classes](#8-classes)
9. [Functions](#9-functions)
10. [`@start` and `@update`](#10-start-and-update)
11. [Event Handlers](#11-event-handlers)
12. [Conditionals](#12-conditionals)
13. [Loops](#13-loops)
14. [Exiting a Loop or the Script](#14-exiting-a-loop-or-the-script)
15. [Error Handling](#15-error-handling)
16. [Operators](#16-operators)
17. [The `json` Type](#17-the-json-type)
18. [Casting and Changing Type](#18-casting-and-changing-type)
19. [Built-in Functions](#19-built-in-functions)
20. [Cores and Plugins](#20-cores-and-plugins)
21. [Full Example](#21-full-example)
22. [Known Limitations](#22-known-limitations)

---

## 1. General Structure

```kli-s-s
i:system;                  // import a core
a:discord;                 // import a plugin

#int counter 0;            // global variable

&Player { ... }            // class

$helper(){ ... }           // function

@start { ... }             // runs once at startup
@update { ... }            // runs repeatedly, in the update loop

@cons.imput(msg){ ... }    // event handler
```

## 2. Comments

Single-line only: `// text to the end of the line`

## 3. Imports: Cores and Plugins

```kli-s-s
i:system;
i:system,math;

a:discord;
```

Keyword, `:`, a comma-separated list of names, `;`. Available cores:
`system` ([`kss-system.md`](kss-system.md)), `math` ([`kss-math.md`](kss-math.md)),
`render` ([`kss-render.md`](kss-render.md)).
Available plugins: `discord` ([`kss-discord.md`](kss-discord.md), a stub).
More on the import mechanism itself — [section 20](#20-cores-and-plugins).

## 4. Data Types

| Type       | Keyword        | Literal example       | Notes |
|------------|----------------|-----------------------|-------|
| Integer    | `int`          | `100`, `-5`           | 64-bit |
| Float      | `float`        | `3.14`                | 64-bit (f64) |
| Boolean    | `bool`         | `true`, `false`       | |
| String     | `string`/`str` | `"text"`               | `str` is a short alias |
| Array      | `type[]`       | `[1, 2, 3]`            | trailing comma allowed; an element at an existing index can be changed with `=`, created/deleted with `.#new[i]`/`.#null[i]` |
| ncti       | `ncti`         | from `int`             | see [`kss-math.md`](kss-math.md) |
| json       | `json`         | `{int#"n":1, ...}`     | see [section 17](#17-the-json-type) |

## 5. Variables

```kli-s-s
#int h 100;
#bool ready true;
#float pi 3.14;
#ncti big 0;
#json gg {int#"n":1};

#int[] scores [10, 20, 30];
```

**There must be no `=` in a declaration** — `#int h = 100;` is an error.
The value always comes right after the name: `#int h 100;`.

- Top-level variables are global.
- Variables inside `@start` also behave as global (visible in `@update`
  and event handlers).
- Variables inside `if`/`while`/`for`/`$functions` are local.

Assigning to an existing variable — no `#` and no type: `h = h + 1;`

## 6. Reserved Words

Cannot be used as names for variables/functions/fields:
`int, float, bool, string, str, ncti, json, i, a, start, update, w, f,
return, break, continue, exetr, exets, new, null, e, true, false`.

`new` and `null` are reserved specifically as keywords after `.#`
— see ["Explicit Creation and Deletion"](#17-the-json-type).

`e` is reserved as the catch-block keyword right after the closing `}`
of a `try` block — see ["Error Handling"](#15-error-handling).

Example: `#str string "da";` **will not work** — `string` is itself a
keyword; name the variable something else (`txt`, `s1`, etc.).

## 7. Assignment Sugar

```kli-s-s
#int t 1;
t + 1;      // t = 2   (equivalent to t = t + 1;)
t - 2;      // t = 0
t * 5;      // t = 0
t / 1;      // t = 0

#int a 6;
#int b 3;
a / b;      // a = 2   (a = a / b)
```

Works for `+ - * / %`.

## 8. Classes

```kli-s-s
&Player {
    #int hp 100;
    $heal(){ hp = hp + 10; }
}
```

Currently only parsed — instantiation is not implemented.

## 9. Functions

```kli-s-s
$greet(){ cons.print("Hello"); }   // call: greet();

$v(){
    $px(){ cons.print("x"); }        // nested function
}
                                      // call: $v.px();
$sum(){ return 10; }
```

### Parameters

```kli-s-s
$f(str#test, int#i){
    cons.println(test);
    cons.println(i);
}

f("tupe", 3);
```

Each parameter is `type#name`, multiple parameters separated by commas.
At the call site, arguments are passed as ordinary expressions in the
same order; the number of arguments must match the number of
parameters, otherwise the interpreter panics. As with regular
variables, the parameter's type isn't strictly checked: `float` and
`ncti` cast the incoming value to their own type, the other types
(`int`, `bool`, `string`, `json`) don't check the value at all.

## 10. `@start` and `@update`

```
@start { /* once, at startup */ }
@update { /* repeatedly, in a loop */ }
```

The loop (event polling + `@update`) continues running as long as
there's an `@update`, an active event handler, or a "live" background
entity (e.g. an open window). It stops via `exets`.

## 11. Event Handlers

```kli-s-s
@cons.imput(msg){
    cons.println(msg);
}
```

`@namespace.event(parameter){...}` — the parameter is optional.
Right now the `cons.imput` event (a line of console input) is
registered by the [`system`](kss-system.md) core; other cores can
register their own events the same way.

## 12. Conditionals

```kli-s-s
?(condition){
    // if
} !?(other){
    // else if — can have several
} !{
    // else
}
```

## 13. Loops

```kli-s-s
@w(condition){ ... }                              // while

@f(#int i 0; i ?< 10; i = i + 1){ ... }         // C-style for
```

## 14. Exiting a Loop or the Script

| Statement    | Meaning                                     |
|--------------|----------------------------------------------|
| `break;`     | exit the current loop                         |
| `exetr;`     | the same thing                                |
| `continue;`  | next iteration                                |
| `exets;`     | immediately terminate the whole script        |
| `return;`    | exit a function (optionally with a value)     |

## 15. Error Handling

```kli-s-s
%{
    #int x (10 / 0);
} e(err){
    cons.println(err);
}
```

`%{...}` is a `try` block. If the interpreter panics inside it
(division by zero, a type mismatch, accessing a nonexistent
field/index, etc.), execution of the block is aborted, the panic is
suppressed, and control moves to `e(name){...}` — the `catch` block.
`name` is declared there as an ordinary `string` variable holding the
error text.

If there was no panic inside `%{...}`, the `e(...)` block is skipped.

The syntax is fixed: right after the closing `}` of the `%{...}` block
there must be `e(name){...}` — nothing else can go there.

## 16. Operators

**Arithmetic:** `+ - * / %` (work for `int`/`float`/`ncti`; `ncti` has no `/` yet)

**Comparisons:**

| `?=` equal | `!=` not equal | `?>` greater | `?<` less |
|---|---|---|---|
| `?>=` greater-or-equal | `?<=` less-or-equal | `!>` not greater | `!<` not less |
| `!>=` not greater-or-equal | `!<=` not less-or-equal | | |

**Logic:** `|` — and, `||` — or

```kli-s-s
?(msg ?= "e" || msg ?= "x"){ exets; }
```

**Unary:**

| `!expression` | inverted truthiness check (logical NOT) |
|---|---|
| `?expression` | truthiness check (no inversion) |
| `-expression` | arithmetic negation |

```kli-s-s
#int aa 11;
#bool rr;
rr = ?aa#int;   // true  (aa is truthy, since it's not 0)
rr = !aa#int;   // false
```

## 17. The `json` Type

An object with fields — the field type is **optional**:

```kli-s-s
#json gg {int#"number":1, bool#"wat":true, str#"string":"eeee"};   // typed
#json plain {"int":1, "bool":true, "str":"string", "obj":{}, "arr":[]};  // untyped
```

Literal format: `{ [type#]"key":value, ... }` — if a type is given
(`int#"key":value`), the field is typed and checked on ordinary
assignment; if there's no type (`"key":value`), the field accepts a
value of any type with no check.

### Lists

`json` can also hold a list (it reuses the ordinary array syntax):

```kli-s-s
#json test [1, true, "string", {}, []];
```

### Reading a Field

```kli-s-s
gg.number + 1;     // read as int, arithmetic works as usual
```

### Ordinary Assignment

```kli-s-s
gg.number = 5;       // fine — the field is typed as int, 5 is also int
gg.number = "str";   // PANIC: the field is typed as int, the value is a string

plain.int = "anything"; // fine — the field "int" was declared WITHOUT a type, no check
```

### Changing a Field's Type — `.#type =`

```kli-s-s
gg.number.#str = "str";   // changes both the type and the value of field number
```

### Adding a New Field — `.#new =`

```kli-s-s
gg.#new = int#"hp":20;    // adds a typed field hp (int, 20)
gg.#new = "note":"ok";    // adds an untyped field
```

### Explicit Creation and Deletion — `.#new` / `.#null`

A more general mechanism that works on **any path** (a variable,
`.field`, `[index]`, in any combination) — for json object fields and
array elements:

```kli-s-s
#json mas [[1,2,3],{"n":"6"}];

mas[0].#new[3] = 4;    // create an array element at an index
mas[1].#new.f = "7";   // create an object field
mas[1].#null.n;        // delete an object field

cons.println(mas);     // [[1, 2, 3, 4], {f: 7}]
```

| Syntax | What it does |
|---|---|
| `path.#new.key = value;` | create (or overwrite) a json object field |
| `path.#new[index] = value;` | create (or overwrite) an array element; if `index` is beyond the current length, the array is padded with `null` up to the needed size |
| `path.#null.key;` | delete a json object field (panics if the field doesn't exist) |
| `path.#null[index];` | delete an array element by index, shifting the rest down (panics if the index is out of bounds) |

Important: navigating the path itself never creates anything
automatically — `mas[5]` on an array of length 3 will panic, as
usual. Only the last, explicitly given `#new`/`#null` step creates or
deletes.

Also, ordinary `=` assignment now works for array elements by index
too — but **only for already-existing** indices, without extending:

```kli-s-s
mas[0][1] = 99;     // fine, index 1 already exists
mas[0][10] = 1;      // PANIC — index out of bounds, use .#new[10] = 1;
```

### Changing the Type of an Ordinary Variable

The same `.#type =` also works for plain variables, not just json fields:

```kli-s-s
#str txt "da";
txt.#int = 10;    // txt now holds int(10)
```

## 18. Casting and Changing Type

Three constructs that are easy to confuse:

| Syntax | What it does | Type check |
|---|---|---|
| `expression#type` | **cast** — evaluates the expression and converts the result to another type, returns a new value | — |
| `expression.#type` (not at the start of a statement) | the same cast, just written with a dot — can be used anywhere in an expression (RHS of an assignment, a function argument, etc.) | — |
| `name[.field].#type = value;` (the whole statement) | **change type in place** — replaces the type and value of the variable/field | not required |
| `name[.field] = value;` | ordinary assignment | type must match only for typed json fields (`type#"key"`); untyped fields and ordinary variables — no check |

```kli-s-s
#int rr 11;
#str dd "12";

dd = rr#str;    // "11" — int turned into a string
rr = dd#int;    // 12   — the string is actually parsed into a number
dd = rr.#str;   // same thing as rr#str
```

Important: `name.#type` behaves differently depending on position.
If it's **the whole statement** and `= value;` immediately follows —
it's changing the variable's/field's type in place (see
[section 17](#17-the-json-type)). If `.#type` appears **inside**
another expression (not as its own statement) — it's an ordinary
cast, leaving the original variable untouched:

```kli-s-s
#str txt "da";

txt.#int = 10;       // change type in place: txt now holds int(10)
#int copy;
copy = txt.#int;      // ordinary cast: copy = int, txt is untouched
```

`name.#type;` **on its own, without an assignment**, as its own
statement — doesn't work (the parser expects `=` right after the type
and panics if it's missing). Use `.#type` either as a cast inside
another expression, or as a type change with an explicit
`= value;`.

### Casting to `json`

For a string, casting to `#json` splits it into an array of characters:

```kli-s-s
#str test "qwe 123";
#json test1;
test1 = test.#json;   // ["q","w","e"," ","1","2","3"]
```

For values that are already an array or a json object, casting to
`json` doesn't change anything (returns the same value).

## 19. Built-in Functions

Always available, without `i:`/`a:` (loaded unconditionally by the
interpreter, unlike core/plugin functions): `str.len("hello")`,
`arr.len([1,2,3])`.

## 20. Cores and Plugins

A core or a plugin is a module on the Rust side that a script imports
via `i:name;` (core) or `a:name;` (plugin) and thereby gains access to
its functions/events in their own namespaces (`cons.*`, `math.*`,
etc. — see [section 3](#3-imports-cores-and-plugins)).

The difference between them is purely organizational: cores are part
of the interpreter's main distribution, plugins are optional add-ons.
Import syntax and how you call the functions are the same.

Documentation for each specific core/plugin lives in its own file:

| Module | Type | Import | Documentation |
|---|---|---|---|
| `system` | core | `i:system;` | [`kss-system.md`](kss-system.md) |
| `math` | core | `i:math;` | [`kss-math.md`](kss-math.md) |
| `render` | core | `i:render;` | [`kss-render.md`](kss-render.md) |
| `discord` | plugin | `a:discord;` | [`kss-discord.md`](kss-discord.md) |

## 21. Full Example

```kli-s-s
i:system;
i:math;

@start {
    #json player {int#"hp":100, "name":"Hero"};   // hp is typed, name is not

    ?(player.hp ?> 50){
        cons.println("OK");
    } !{
        cons.println("Low health");
    }

    #int roll (rand.rdom(1, 6));
    cons.print("Dice roll: ");
    cons.println(roll);

    player.hp = player.hp - roll;   // hp stays int — types match
    player.name = "Champion";       // name has no type — anything goes
}

@cons.imput(text){
    ?(text ?= "e" || text ?= "x"){
        exets;
    }
}
```

The example uses `rand.rdom`, `cons.print`/`cons.println`, and the
`cons.imput` event from the [`system`](kss-system.md) core — see
`i:system;` at the top.

## 22. Known Limitations

- Functions have no default parameter values and no overloads — the
  number of arguments at a call site must exactly match the
  declaration.
- Classes have no instantiation — declaration only.
- `json`: casting to `json` is only meaningful for strings (it splits
  them into an array of characters, see
  [section 18](#18-casting-and-changing-type)); for other types and
  for casting to an arbitrary class, it just returns the original
  value unchanged.
- Scripts cannot register their own events/ticks — only cores/plugins
  on the Rust side can do that.

Limitations of specific cores and plugins (`ncti`, windows, `discord`)
are in their own documentation files.
