#!/bin/sh
set -e
set -x 

echo "#########################################################"
echo "############# ENTRYPOINT SCRIPT STARTED #################"
echo "Current environment variables:"
printenv
echo "---------------------------------------------------------"

# Simple wait function
wait_for() {
    echo "Waiting for $1 to be ready..."
    while ! nc -z $1 $2; do 
        sleep 1
    done
    echo "$1 is ready."
}

# Get DB host and port from URI
DB_HOST=$(echo $PARSE_SERVER_DATABASE_URI | sed -n 's/mongodb:\/\/\([^:]*\):\([0-9]*\)\/.*/\1/p')
DB_PORT=$(echo $PARSE_SERVER_DATABASE_URI | sed -n 's/mongodb:\/\/\([^:]*\):\([0-9]*\)\/.*/\2/p')

if [ -z "$DB_HOST" ] || [ -z "$DB_PORT" ]; then
    echo "Error: Could not parse DB_HOST or DB_PORT from PARSE_SERVER_DATABASE_URI: $PARSE_SERVER_DATABASE_URI"
    exit 1
fi

wait_for $DB_HOST $DB_PORT

echo "Starting Parse Server in background..."
node ./bin/parse-server \
    --appId ${PARSE_SERVER_APPLICATION_ID} \
    --masterKey ${PARSE_SERVER_MASTER_KEY} \
    --databaseURI ${PARSE_SERVER_DATABASE_URI} \
    --port ${PARSE_SERVER_PORT} \
    --serverURL http://localhost:${PARSE_SERVER_PORT}/parse \
    --host ${PARSE_SERVER_HOST} > /tmp/parse-server.log 2>&1 &
PARSE_PID=$!
echo "Parse Server process started with PID: $PARSE_PID (logs at /tmp/parse-server.log)"

PARSE_PORT_FROM_ENV=${PARSE_SERVER_PORT}
if [ -z "$PARSE_PORT_FROM_ENV" ]; then
    echo "Error: PARSE_SERVER_PORT environment variable is not set. Exiting."
    exit 1
fi

HEALTH_URL="http://parse-server-ephemeral:$PARSE_PORT_FROM_ENV/parse/health" 
echo "############# STARTING HEALTH CHECK LOOP #################"
echo "Waiting for Parse Server health check at $HEALTH_URL..."
max_retries=30
retry_count=0
while [ $retry_count -lt $max_retries ]; do
    echo "Health check attempt $((retry_count + 1))/$max_retries for $HEALTH_URL..."
    if curl --fail --verbose --max-time 5 $HEALTH_URL; then
        echo "Parse Server is healthy at $HEALTH_URL."
        break
    else
        echo "Health check failed for attempt $((retry_count + 1)). HTTP status or curl error above."
    fi
    retry_count=$((retry_count + 1))
    if [ $retry_count -lt $max_retries ]; then
        echo "Waiting 2 seconds before next health check attempt..."
        sleep 2
    fi
done
echo "############# HEALTH CHECK LOOP FINISHED ###############"

if [ $retry_count -ge $max_retries ]; then
    echo "Error: Parse Server did not become healthy at $HEALTH_URL after $max_retries attempts."
    echo "Killing Parse Server PID $PARSE_PID due to health check timeout."
    kill $PARSE_PID || echo "Failed to kill PID $PARSE_PID, it might have already exited."
    exit 1
fi

node /parse-server/create-admin.js

node /parse-server/test-connection.js

echo "Bringing Parse Server process (PID: $PARSE_PID) to foreground..."
echo "#########################################################"
echo "############## ENTRYPOINT SCRIPT FINISHING ##############"
echo "#########################################################"
wait $PARSE_PID
echo "Parse Server process $PARSE_PID exited."
