#! /usr/bin/env bash

set -euo pipefail
# set -x
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
IMAGE=megalinter/megalinter:v5

if [ -x "$(command -v nerdctl)" ]; then
  nerdctl run --pull always --rm -it -v "${SCRIPT_DIR}:/tmp/lint:rw" "$IMAGE"
elif [ -x "$(command -v docker)" ]; then
  docker run --pull always --rm -it -v /var/run/docker.sock:/var/run/docker.sock:rw -v "${SCRIPT_DIR}:/tmp/lint:rw" "$IMAGE"
else
  echo "runner not found: docker or nerdctl"
  exit 1
fi
