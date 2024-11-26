#!/bin/bash

# For simple container monitoring, will log stats every x seconds

MAX_LINES=1440
SERVICES=("dbc-bot" "bracket" "images-server" "postgresql")

while true; do
        for service in ${SERVICES[@]}; do
                log_file="./logs/${service}.stats.log"
                docker stats --no-stream | grep -e 'CONTAINER' -e ${service} | ts '[%Y-%m-%d %H:%M]' > ${log_file}_
                touch ${log_file}
                grep $service $log_file | head -${MAX_LINES} >> ${log_file}_
                mv ${log_file}_ ${log_file}
        done
        sleep 10;
done
