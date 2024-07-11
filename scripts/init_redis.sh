#!/usr/bin/env bash
# For the script to work with Github Actions, leave the path as it is

set -x
set -eo pipefail

# If a Redis container is running, print instructions to kill it and exit
RUNNING_CONTAINER=$(docker ps --filter 'name=redis' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
    echo >&2 "There is a Redis container running. Please kill it before running this script."
    echo >&2 "You can run 'docker kill $RUNNING_CONTAINER' to stop it."
    exit 1
fi

# Launch Redis with Docker
docker run \
    --name "redis_$(date '+%s')" \
    -p 6379:6379 \
    -d \
    redis

echo >&2 "Redis is up and running on port 6379!"
