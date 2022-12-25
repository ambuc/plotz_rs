# `svg-splitter`

SvgSplitter is a tool for splitting very large SVG files which already have inherent groupings.

## Usage

Imagine a very large SVG file with some linesets, which might be the output of a tool like Blender or Inkscape:

```svg
// in.svg
<?xml ...?>
<svg xmlns="http://www.w3.org/2000/svg" ... >
    <g id="ViewLayer_LineSet group1" ... > # group 1
        <g inkscape:groupmode="layer" id="strokes" inkscape:label="strokes">
            <path fill="none" d="..." /> # path 1.1
            <path fill="none" d="..." /> # path 1.2
            ...
        </g>
    </g>
    <g id="ViewLayer_LineSet group2" ... > # group 1
        <g inkscape:groupmode="layer" id="strokes" inkscape:label="strokes">
            <path fill="none" d="..." /> # path 2.1
            <path fill="none" d="..." /> # path 2.2
            ...
        </g>
    </g>
    ...
</svg>
```

Run SvgSplitter with the input file and an output prefix:

```bash
cargo run -p svg_splitter -- --input "/path/to/in.svg" --output "/path/to/out"
```

And it will generate output SVGs named `out_group1.svg`, `out_group2.svg`, ... like:

```svg
// out_group1.svg
<?xml ...?>
<svg xmlns="http://www.w3.org/2000/svg" ... >
    <g id="ViewLayer_LineSet group1" ... > # group 1
        <g inkscape:groupmode="layer" id="strokes" inkscape:label="strokes">
            <path fill="none" d="..." /> # path 1.1
            <path fill="none" d="..." /> # path 1.2
            ...
        </g>
    </g>
</svg>
```

```svg
// out_group2.svg
<?xml ...?>
<svg xmlns="http://www.w3.org/2000/svg" ... >
    <g id="ViewLayer_LineSet group2" ... > # group 2
        <g inkscape:groupmode="layer" id="strokes" inkscape:label="strokes">
            <path fill="none" d="..." /> # path 2.1
            <path fill="none" d="..." /> # path 1.2
            ...
        </g>
    </g>
</svg>
```

SvgSplitter also has an optional `--split=n` flag which will ensure that no output SVG has more than N path elements, overflowing the remainder into `out_group1_1.svg`, `out_group1_2.svg`, etc.
