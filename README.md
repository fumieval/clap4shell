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

The document shape follows [clap-serde](https://docs.rs/clap-serde/latest/clap_serde/). That crate stops at clap 3, so clap4shell reads the document itself and builds a clap 4 command.

Flags are counted: `-vv` becomes `verbose=2`, and an omitted flag is `0`. `action: set_true` is a boolean printed as `0` or `1`. Values are single-quoted so they can be `eval`'d, including values that contain quotes.

clap 4 fields are available in the same document:

- `action`: `set`, `append`, `set_true`, `set_false`, `count`, `help`, `version` (`arg_action` is also accepted)
- `default_value`, `default_values`, `default_missing_value`
- `num_args`: `1`, `1..`, `0..=1`, `2..=4` (`min_values`, `max_values`, and `number_of_values` still work)
- `value_delimiter`, and `value_hint` (`file_path`, `dir_path`, `any_path`, and the other clap hints)
- `value_parser`: `string`, `os_string`, `bool`, `i64`, `u64`, `f64`, and the other integer widths. `bool` is printed as `0` or `1`
- `required`, `requires`, `conflicts_with`, `global`, `env`, `allow_hyphen_values`, `allow_negative_numbers`
- `groups` for mutually exclusive options
- `possible_values` with `ignore_case`, plus `aliases` and `hide`

`args`, `subcommands`, and `groups` accept either a list of single-key maps or a map. An omitted positional is left unset; set `required: true` when it must be present.

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

`clap4shell-completion <shell> -o <path>` generates an autocompletion script for `<shell>` and writes it to `path`. `<shell>` is `bash`, `elvish`, `fish`, `powershell` (`pwsh`), or `zsh`. When the shell is omitted, it is taken from `$SHELL`.

```sh
clap4shell clap4shell-completion bash -o ./bash-completion.sh < definition.yaml
```

When "-o" is omitted, it prints an echo command that prints the script, followed by `exit 0`. Therefore this feature can be used transparently even if the output of `clap4shell` is passed to `eval`.

```
./foo.sh clap4shell-completion > ~/.zsh/_foo.sh
```

Notes
----

Definitions are built with clap 4's builder. clap removed `Command::from_yaml`. Parse errors are clap's own messages and are written to stderr, so a script that `eval`s stdout does not execute them. `--help` and `--version` also go to stderr and exit nonzero, which stops a `set -e` script after the text is shown.