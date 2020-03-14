urgeopts: standalone clap wrapper
----

`urgeopts` is a replacement for `getopt` based on [clap](https://github.com/clap-rs/clap).

`urgeopts` takes option descriptors as a YAML document from stdin, and parse arguments passed to it, and prints the result which can be `eval`'d. See example.sh for concrete usage.

I intentionally didn't use `clap::App::from_yaml` because the YAML representation tend to be redundant, and hard to diagnose panics.