#!/bin/bash

# For simple container monitoring, logs stats every X seconds
MAX_LINES=1440
SERVICES=("dbc-bot" "bracket" "images-server" "postgresql")
LOG_DIR="./logs"
PROCESS=$(basename "$0")
INTERVAL=10

# Ensure log directory exists
mkdir -p "${LOG_DIR}"

function clean_up() {
  rv=$?
  echo "Exit code received: $rv"
  if [ $rv -eq 137 ]; then
    echo "Manually killed"
  elif [ $rv -eq 0 ]; then
    echo "Exited smoothly"
  else
    echo "Non-zero exit code: $rv"
  fi
  exit $rv
}

trap clean_up EXIT

# Kill any previously running instance of this script
pgrep -fi "$PROCESS" | grep -v "$$" | xargs -r kill -9

# Start monitoring
while true; do
  for service in "${SERVICES[@]}"; do
    log_file="${LOG_DIR}/${service}.stats.log"
    temp_file="${log_file}_"

    # Get stats for the service
    docker stats --no-stream | grep -e 'CONTAINER' -e "$service" | ts '[%Y-%m-%d %H:%M]' > "$temp_file"

    # Rotate logs
    touch "$log_file"
    tail -n $MAX_LINES "$log_file" >> "$temp_file"
    mv "$temp_file" "$log_file"
  done
  sleep $INTERVAL
done
