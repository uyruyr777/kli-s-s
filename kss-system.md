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

## Windows — `window`

| Function | Description |
|---|---|
| `window.open(xh, yh, x, y, name, sysid)` | open a real window (X11/Windows via `minifb`) with width `xh`, height `yh`, at position `(x, y)`, titled `name`; `sysid` is an integer id the script makes up itself and uses afterward to refer to this window |
| `window.clos(sysid)` | close the window with the given `sysid` |

Exactly 6 arguments for `open` and 1 for `clos`, otherwise a panic.
`window.open` panics if a window with that `sysid` is already open, or
if actually opening the window at the OS level failed. Returns `bool`
(success). `window.clos` returns `bool` — whether a window with that
`sysid` was found at all.

While a window is open, the interpreter checks on every tick whether
it's been closed (via the close button or the `Escape` key) — no
event is delivered inside the script for this, but the open window
keeps the [`@update` loop](kss.md#10-start-and-update) from ending
until it closes.

Drawing specific pixels/shapes from a script isn't implemented yet —
the window is just filled with a background color.

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

- Drawing inside an open window (pixels, shapes) isn't implemented —
  background fill only.
- `rand.rdom` is not a cryptographic generator, not suitable for
  anything that needs real randomness.
- `file.read` silently returns `""` on any read error (doesn't
  panic) — you can't tell "file is empty" from "file doesn't exist"
  through this function; use `file.exists` for that.
