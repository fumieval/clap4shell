clap4shell: standalone clap wrapper for shell scripts
----

[![clap4shell at crates.io](https://img.shields.io/crates/v/clap4shell.svg)](https://crates.io/crates/clap4shell)

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
bin_name: $(basename $0)
version: "0.0"
about: sample text
author: Fumiaki Kinoshita <fumiexcel@gmail.com>
args:
  - verbose:
      help: verbose output
      short: v
      long: verbose
  - host:
      takes_value: true
      long: host
      value_name: <HOST>
      help: 'host name'
  - mode:
      long: mode
      help: option with a certain set of possible values
      possible_values: [ herbivore, carnivore, omnivore ]
  - cmd:
      help: command
  - arg:
      help: command arguments
      multiple_values: true
subcommands:
  - ls:
      about: Display a list of entities
      args:
        - entity:
            multiple_values: true
```

The schema is based on [clap_serde](https://docs.rs/clap-serde/latest/clap_serde/).

The typical usage is to embed option definitions in a shell heredoc, pass all arguments to `clap4shell` and `eval` its output:

```bash
eval "$(clap4shell "$@" <<EOT
...
EOT
)"
```

Subcommands are bound as positional parameters (`$1, $2, ...`).

Completion
----

`clap4shell-completion <shell> -o <path>` subcommand generates an autocompletion script for `<shell>` and writes the script to `path`.

```sh
clap4shell clap4shell-completion bash -o ./bash-completion.sh < definition.yaml
```

When "-o" is omitted, it prints an echo command that prints the script, followed by `exit 0`. Therefore this feature can be used transparently even if the output of `clap4shell` is passed to `eval`.

```
./foo.sh clap4shell-completion > ~/.zsh/_foo.sh
```

Notes
----

I intentionally avoided using `clap::App::from_yaml` because the redundancy of the YAML representation and difficulty of debugging panics.
The YAML interface is being deprecated: https://github.com/clap-rs/clap/issues/3087