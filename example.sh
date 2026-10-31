#!/usr/bin/env bash

set -ue

eval "$(clap4shell "$@" <<EOT
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
  - token:
      long: token
      help: this option can be specified via an environment variable too
      env: CLAP4SHELL_TOKEN
      takes_value: true
  - force:
      long: force
      action: set_true
      help: overwrite existing output
  - config:
      long: config
      value_name: PATH
      value_hint: file_path
      help: configuration file
  - format:
      long: format
      possible_values: [ text, json ]
      default_value: text
      help: output format
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
EOT
)"

echo "subcommands: $@"

# Print all variables
declare -p | tail