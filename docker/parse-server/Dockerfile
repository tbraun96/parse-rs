# docker/parse-server/Dockerfile
FROM parseplatform/parse-server:latest

# Install curl for health checks if needed within the entrypoint script
# The base image already includes node and npm/npx
USER root
# Use apk (Alpine package manager) instead of apt-get
RUN apk update && apk add --no-cache curl && rm -rf /var/cache/apk/*

# Copy and set permissions for the custom entrypoint script AS ROOT
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Explicitly create the group and user
# -S creates a system user/group (no password, home dir, etc.)
RUN addgroup -S parse && adduser -S -G parse parse

# Create the working directory and set ownership before switching user
RUN mkdir -p /parse-server && chown -R parse:parse /parse-server

# Now switch to the non-root parse user
USER parse

WORKDIR /parse-server

# Copy the initialization and test scripts as the parse user
COPY --chown=parse:parse create-admin.js .
COPY --chown=parse:parse test-connection.js .

# Acquire the env var PARSE_SERVER_PORT from the docker-compose.yml
ARG PARSE_SERVER_PORT
ENV PARSE_SERVER_PORT=${PARSE_SERVER_PORT}

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# Expose the default Parse Server port from the ENV
EXPOSE ${PARSE_SERVER_PORT}

# The default CMD from the base image is usually `npm start` or similar,
# which our entrypoint script will call eventually.
CMD ["npm", "start"] 
