#!/usr/bin/env bash
set -e
function script_echo() {
  echo "unleash-actix-example: $1"
}

function generate_buildinfo() {
  output=${1}
  trigger_event=${2}
  self_git_sha=$(git rev-parse --short=7 HEAD)

  cat <<EOT > ${output}
  {
    "commits": [
      {
        "slug": "unleash/unleash-actix-example",
        "id": "${self_git_sha}"
      }
    ],
    "project": "unleash-actix-example",
    "trigger": {
      "type": "commit",
      "source": "unleash/unleash-actix-example",
      "commitIds": ["${self_git_sha}"]
    },
    "docker": {
      "image": "${DOCKER_IMAGE}",
      "tag": "sha-${self_git_sha}"
    },
    "unixTimestamp": "$(date +%s)"
  }
EOT
}
generate_buildinfo buildinfo.json
script_echo "$(cat buildinfo.json)"

curl -X POST -H "Content-Type: application/json" https://sandbox.getunleash.io/pipeline/build_info -d @buildinfo.json