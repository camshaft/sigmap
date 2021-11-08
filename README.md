# sigmap
Maps process signals from one code to another

## Installation

```bash
$ cargo install sigmap --force
```

## Usage

```bash
# Maps TERM signals to INT
$ sigmap TERM:INT ./my-process arg1 arg2

# Maps a list of signals
$ sigmap TERM:INT HUP:INT ./my-process arg1 arg2
```
