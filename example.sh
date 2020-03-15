#!/bin/bash

set -ue

eval $(urgeopts "$@" <<EOT
name: example
bin_name: $0
version: "1.0"
about: sample text
author: Fumiaki Kinoshita <fumiexcel@gmail.com>
settings:
  - ArgRequiredElseHelp
flags:
  verbose:
    help: verbose output
    short: v
    long: verbose
opts:
  host:
    help: host name
    long: host
  mode:
    long: mode
    help: option with a certain set of possible values
    possible_values: [ herbivore, carnivore, omnivore ]
args:
  - name: cmd
    help: command
    required: true
  - name: arg
    help: command arguments
    multiple: true
EOT
)

# Print all variables
declare -p | tail