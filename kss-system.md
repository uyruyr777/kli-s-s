# The `system` Core

Part of the interpreter's main distribution. Imported via `i:system;`
(see the [main documentation](kss.md#3-imports-cores-and-plugins)).

```kli-s-s
i:system;
```

## Console — `cons`

| Function | Description |
|---|---|
| `cons.print(x, ...)` | output without a trailing newline; accepts any number of arguments, prints them one after another with no separator |
| `cons.println(x, ...)` | the same, but with a trailing newline |

Both accept zero or more arguments — each is printed via `Display`,
with no space between them: `cons.print("a", "b")` prints `ab`.

## Files — `file`

| Function | Description |
|---|---|
| `file.read(path)` | return the file's contents as a `string`; if the file doesn't exist or reading fails, returns an empty string, no panic |
| `file.write(path, text)` | write `text` to the file, overwriting it; panics on a write error |
| `file.exists(path)` | `bool`, whether the path exists |

All three take exactly the stated number of arguments (1, 2, and 1
respectively) — otherwise a panic with a signature hint.

## Time — `time`

| Function | Description |
|---|---|
| `time.sleep(ms)` | block script execution for `ms` milliseconds |

## Random Numbers — `rand`

| Function | Description |
|---|---|
| `rand.rdom(min, max)` | a random integer from `[min, max]`, **inclusive** |

Panics if `max < min`. The generator is a custom one (not
cryptographic), based on system time and a call counter.

## The `cons.imput` Event

```kli-s-s
@cons.imput(msg){
    cons.println(msg);
}
```

Fires when a line is entered on the console (reads stdin line by
line). `msg` is the entered line as a `string`, without the trailing
newline.

## Limitations

- `rand.rdom` is not a cryptographic generator, not suitable for
  anything that needs real randomness.
- `file.read` silently returns `""` on any read error (doesn't
  panic) — you can't tell "file is empty" from "file doesn't exist"
  through this function; use `file.exists` for that.

Windows and drawing (`window.*`) moved to the
[`render`](kss-render.md) core.
