clap4shell: standalone clap wrapper for shell scripts
----

[![clap4shell at crates.io](https://img.shields.io/crates/v/clap4shell.svg)](https://crates.io/crates/clap4shell)
[![clap4shell at docs.rs](https://docs.rs/clap4shell/badge.svg)](https://docs.rs/clap4shell)


`clap4shell` is a replacement for `getopt` based on [clap](https://github.com/clap-rs/clap).

`clap4shell` takes option descriptors as a YAML document from stdin, then prints the parsed result in newline-delimited `key=value` format which can be `eval`'d. See example.sh for concrete usage.

Installation
----

```sh
nix profile install github:fumieval/clap4shell
```

Usage
----

Configurations look like

```yaml
name: example
bin_name: $0
version: "1.0"
about: sample text
author: Fumiaki Kinoshita <fumiexcel@gmail.com>
# Flags take no arguments.
# Number of occurences will be reported (e.g. verbose=1)
flags:
  verbose:
    help: verbose output
    short: v
    long: verbose
# Options arguments are collected as newline-separated list of values.
opts:
  # usage syntax is also supported
  # cf. https://docs.rs/clap/2.33.0/clap/struct.Arg.html#syntax
  host: -h --host <HOST> 'host name'
  mode:
    long: mode
    help: option with a certain set of possible values
    possible_values: [ herbivore, carnivore, omnivore ]
# Positional arguments
args:
  - name: cmd
    help: command
    required: true
  - name: arg
    help: command arguments
    multiple: true
subcommands:
  ls: #The name will be assigned to $subcommand
    about: Display a list of entities
    args:
    - name: name
      multiple: true
```

The typical usage is to embed option definitions in a shell heredoc, pass all arguments to `clap4shell` and `eval` its output:

```bash
eval "$(clap4shell parse "$@" <<EOT
...
EOT
)"
```

Completion
----

`clap4shell-completion <shell> -o <path>` subcommand generates an autocompletion script and writes the script to `path`.

```sh
clap4shell clap4shell-completion bash -o ./bash-completion.sh < definition.yaml
```

Note that clap does not support completion for subcommands yet (clap panics for some reason)

Notes
----

I intentionally avoided using `clap::App::from_yaml` because the redundancy of the YAML representation and difficulty of debugging panics.
The YAML interface is being deprecated: https://github.com/clap-rs/clap/issues/3087