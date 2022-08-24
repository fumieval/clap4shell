#!/usr/bin/env bash

set -ue

eval "$(clap4shell parse "$@" <<EOT
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
# subcommands:
#  ls:
#    about: Display a list of entities
#    args:
#    - name: name
#      multiple: true
EOT
)"

# Print all variables
declare -p | tail