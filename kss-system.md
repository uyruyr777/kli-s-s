# The `system` Core

Part of the interpreter's main distribution. Imported via `i:system;`
(see the [main documentation](README.md#3-imports-cores-and-plugins)).

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

## Filesystem — `fs`

| Function | Description |
|---|---|
| `fs.imp(path)` | import the file's contents as a `string`; **panics** if the file doesn't exist or reading fails (unlike the old `file.read`) — wrap in a [`try`/`catch`](README.md#15-error-handling) if the file may be missing |
| `fs.write(path, text)` | write `text` to the file, overwriting it; panics on a write error, returns `bool` (`true`) on success |
| `fs.exists(path)` | `bool`, whether the path exists |
| `fs.kss(path)` | parse and run another `.kss` file **inline, in the current process**, sharing this script's global scope — see below |

All four take exactly the stated number of arguments (1, 2, 1, and 1
respectively) — otherwise a panic with a signature hint.

`fs.imp` returns a plain `string` — cast it with `.#type` if you need
something else:

```kli-s-s
#str txt;
txt = fs.imp("text.txt").#str;
```

### `fs.kss(path)` — running another script inline

```kli-s-s
i:system;

@start {
    #int shared 5;
    fs.kss("helper.kss");   // helper.kss can read/write `shared`,
                             // and its own functions/variables become
                             // part of this script from this point on
}
```

The target file is lexed and parsed like any `.kss` program and then
merged into the **currently running** interpreter, not spawned as a
separate process:

- its `i:`/`a:` imports are loaded into the same runtime (safe to
  import something already imported);
- its top-level variables are declared into whichever scope was
  active at the `fs.kss(...)` call site (global scope if called from
  `@start`, a local scope if called from inside a function/block);
- its functions and event handlers are registered and become callable
  from this script too;
- its `@start` body (if any) runs immediately;
- its `@update` body (if any) starts running on every tick from then
  on, alongside this script's own `@update`.

Limitation: if `fs.kss` is called **after** the main update loop has
already started (from `@update` or an event handler, not from
`@start`) and the imported file pulls in a *new* core/plugin that
registers its own tick function or event source (for example,
`i:render;` opening its first window), that new tick/event source
only takes effect on the *next* run of the script, not the one
currently executing — newly declared `@update` bodies do take effect
immediately, this limitation is specifically about brand-new
tick/event hooks from a kernel/plugin that wasn't already imported.

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
- `fs.imp`/`fs.kss` panic on any read error — there's no silent
  fallback anymore (the old `file.read` used to return `""`); use
  `fs.exists` to check first, or [`try`/`catch`](README.md#15-error-handling)
  to handle the panic.
- See the note under `fs.kss` above for the one case where a
  dynamically-imported kernel/plugin's tick/event hooks don't take
  effect until the next run.

Windows and drawing (`window.*`) moved to the
[`render`](kss-render.md) core.
