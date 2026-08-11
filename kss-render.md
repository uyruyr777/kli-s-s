# The `render` Core

Part of the interpreter's main distribution. Imported via `i:render;`
(see the [main documentation](kss.md#3-imports-cores-and-plugins)).

```kli-s-s
i:render;
```

All windowing and drawing lives in the `window` namespace. A real OS
window is opened via `minifb` (X11/Windows).

## Opening and Closing

| Function | Description |
|---|---|
| `window.open(xh, yh, x, y, name, sysid)` | open a window with width `xh`, height `yh`, at position `(x, y)`, titled `name`; `sysid` is an integer id the script makes up itself and uses afterward to refer to this window |
| `window.clos(sysid)` | close the window with the given `sysid` |

Exactly 6 arguments for `open`, 1 for `clos`, otherwise a panic.
`window.open` panics if a window with that `sysid` is already open, or
if actually opening the window at the OS level failed. Returns `bool`
(success). `window.clos` returns `bool` — whether a window with that
`sysid` was found at all.

While a window is open, the interpreter checks on every tick whether
it's been closed (via the close button or the `Escape` key) — no
event is delivered inside the script for this, but the open window
keeps the [`@update` loop](README.md#10-start-and-update) from ending
until it closes. The window is redrawn from its buffer every tick
automatically — there's no separate "present"/"flip" call.

## Drawing

All colors are three separate `int` arguments `r, g, b` (each
`0`–`255`, out-of-range values are clamped — no packed hex color
value). All coordinates are in pixels, `(0, 0)` is the top-left
corner. Drawing outside the window's bounds is silently clipped, no
panic.

| Function | Description |
|---|---|
| `window.clear(sysid, r, g, b)` | fill the whole window with a color |
| `window.pixel(sysid, x, y, r, g, b)` | set a single pixel |
| `window.rect(sysid, x, y, w, h, r, g, b)` | filled rectangle |
| `window.rectLine(sysid, x, y, w, h, r, g, b)` | rectangle outline |
| `window.line(sysid, x1, y1, x2, y2, r, g, b)` | a line between two points |
| `window.circle(sysid, cx, cy, radius, r, g, b)` | filled circle |
| `window.circleLine(sysid, cx, cy, radius, r, g, b)` | circle outline |

Every drawing function panics if there's no window with the given
`sysid`.

```kli-s-s
i:render;
i:system;

@start {
    window.open(320, 240, 100, 100, "Demo", 1);
    window.clear(1, 20, 20, 30);
    window.rect(1, 20, 20, 100, 60, 220, 60, 60);
    window.circleLine(1, 200, 120, 40, 60, 220, 120);
    window.line(1, 0, 0, 319, 239, 255, 255, 255);
}

@update {
    time.sleep(16);
}
```

## Limitations

- No text rendering.
- No image loading — only flat colors and basic shapes.
- `window.rect`/`window.circle` are filled unconditionally; there's no
  alpha/transparency, each draw call fully overwrites the pixels it
  touches.
