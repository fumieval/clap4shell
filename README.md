urgeopts: standalone clap wrapper
----

`urgeopts` is a replacement for `getopt` based on [clap](https://github.com/clap-rs/clap).

`urgeopts` takes option descriptors as a YAML document from stdin, and parse arguments passed to it, and prints the result which can be `eval`'d. See example.sh for concrete usage.

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
  host:
    help: host name
    long: host
    multiple: true
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
```

Usually, you want to embed configurations in a heredoc, pass all arguments to `urgeopts` and `eval` its output:

```bash
eval $(urgeopts "$@" <<EOT
...
EOT
)
```

Notes
----

I intentionally didn't use `clap::App::from_yaml` because the YAML representation tend to be redundant, and hard to diagnose panics.