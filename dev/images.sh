#!/usr/bin/env bash

set -euo pipefail
SCRIPT_DIR="$(
    cd "$(dirname "$0")" >/dev/null 2>&1
    pwd -P
)"
SCRIPT_NAME=$(basename $0)
SCRIPT_PATH="$SCRIPT_DIR/$SCRIPT_NAME"

log-lines() { for a in "$@"; do printf "\n  $a" 1>&2 ;done ;printf "\n" 1>&2 ;}
log-info() { printf "$(tput setaf 2)INFO$(tput sgr0): $1" 1>&2; log-lines "${@:2}"; }
log-warn() { printf "$(tput setaf 3)WARN$(tput sgr0): $1" 1>&2; log-lines "${@:2}"; }
log-error() { printf "$(tput setaf 1)ERROR$(tput sgr0): $1" 1>&2; log-lines "${@:2}"; exit 1; }
highlight() { d="";	for a in "$@"; do printf "$(tput bold)$(tput setaf 5)$d$a$(tput sgr0)";	d=" "; done; }

# Function to print out error message and usage examples
usage() {
    cat 1>&2 <<EOF
Usage: $SCRIPT_NAME NR_OF_IMAGES BASEDIR

Examples:
  $ $SCRIPT_NAME dev

EOF
    (log-error "")
    echo -n "  " 1>&2
}

nr_of_images=${1:?"NR_OF_IMAGES <- missing parameter $(usage)"}
basedir=${2:?"BASEDIR <- missing parameter $(usage)"}

dir=$basedir/$RANDOM
mkdir -p $dir
seq $nr_of_images | xargs -I {} -P 8 wget -q -O $dir/$RANDOM{}.jpg https://source.unsplash.com/1920x1200/\?stars,nature,sea,weather