# `orchestrator`

Orchestrator is a tool for managing long-running prints via
[`axicli`](https://axidraw.com/). Given a set of files, the tool can step
through them and offer to preview (estimate print time) and toggle the pen
before each layer.

## Usage:

```bash
$ cargo run  -p orchestrator -- --glob "/path/to/*.svg" --frame "/path/to/specific.svg"
OK Found Files: ["/path/to/foo.svg", "/path/to/bar.svg", "/path/to/specific.svg"]
    ...will plot frame "/path/to/specific.svg"
    ...and then other layers {"/path/to/foo.svg", "/path/to/bar.svg"}

ACTION Preview /path/to/specific.svg (frame) Y/n
OK Estimated print time: 01:23 (minutes, seconds)
...

ACTION Toggle? Y/n
ACTION Toggle? (again) Y/n
ACTION Print /path/to/specific.svg (frame)? Est. 01:23 Y/n
```