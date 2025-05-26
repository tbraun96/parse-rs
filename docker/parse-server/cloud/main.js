// /Users/nologik/nomyx/parse-rs/docker/parse-server/cloud/main.js

Parse.Cloud.define("hello", async (request) => {
  return "Hello from Cloud Code!";
});

Parse.Cloud.define("echo", async (request) => {
  const message = request.params.message;
  if (!message) {
    // For robust error handling, consider using Parse.Error
    // Example: throw new Parse.Error(Parse.Error.INVALID_JSON, "Missing 'message' parameter.");
    throw new Error("Missing 'message' parameter.");
  }
  return { echoedMessage: message };
});

// Add any other Parse.Cloud.define or Parse.Cloud.job calls here
