const Parse = require('parse/node');

// Use environment variables provided by Docker Compose/container
const APP_ID = process.env.PARSE_SERVER_APPLICATION_ID;
const MASTER_KEY = process.env.PARSE_SERVER_MASTER_KEY;
const REST_API_KEY = process.env.PARSE_SERVER_REST_API_KEY;
const JAVASCRIPT_KEY = process.env.PARSE_SERVER_JAVASCRIPT_KEY;
const PARSE_PORT = process.env.PARSE_SERVER_PORT;
if (!PARSE_PORT) {
    console.error('Error: PARSE_SERVER_PORT is not set');
    process.exit(1);
}
const SERVER_URL = `http://0.0.0.0:${PARSE_PORT}/parse`; // Target localhost inside the container
const TEST_USERNAME = process.env.ADMIN_USERNAME;
const TEST_PASSWORD = process.env.ADMIN_PASSWORD;

// Initialize Parse SDK
if (APP_ID && JAVASCRIPT_KEY && SERVER_URL && MASTER_KEY) { 
    Parse.initialize(APP_ID, JAVASCRIPT_KEY, MASTER_KEY); 
    Parse.serverURL = SERVER_URL;
    console.log('Parse SDK initialized with App ID, JavaScript Key, and Master Key.');
} else {
    console.error('Error: Missing APP_ID, JAVASCRIPT_KEY, MASTER_KEY, or SERVER_URL for Parse SDK initialization.');
    process.exit(1); 
}

async function testParseConnection() {
  console.log('Testing Parse Server connection...');
  
  if (!APP_ID || !MASTER_KEY || !SERVER_URL || !TEST_USERNAME || !TEST_PASSWORD || !REST_API_KEY || !JAVASCRIPT_KEY) { 
    console.error('Error: Missing required environment variables for connection test script.');
    process.exit(1);
  }

  try {
    console.log(`Initialized Parse with:
- App ID: ${APP_ID}
- Server URL: ${SERVER_URL}`);
    
    // Test 1: Basic connection using native fetch
    try {
      console.log('\nTest 1: Testing basic connection...');
      
      // Use global fetch which is available in Node.js v18+
      const health = await fetch(`${SERVER_URL}/health`);
      const healthData = await health.json();
      
      console.log(`Health check response: ${JSON.stringify(healthData)}`);
      
      // The health endpoint returns status: "initialized" which is good
      if (healthData.status === "initialized" || healthData.status === "ok") {
        console.log('✅ Server is initialized and ready');
      } else {
        throw new Error(`Unexpected health status: ${healthData.status}`);
      }
      
      console.log('✅ Basic connection successful');
    } catch (error) {
      console.error('❌ Basic connection failed:', error.message);
    }
    
    // Test 2: User login
    try {
      console.log('\nTest 2: Testing user login...');
      console.log(`Attempting to log in as: ${TEST_USERNAME}`);
      
      const user = await Parse.User.logIn(TEST_USERNAME, TEST_PASSWORD);
      console.log(`✅ Login successful! User ID: ${user.id}`);
      console.log(`Session token: ${user.getSessionToken()}`);
    } catch (error) {
      console.error('❌ Login failed:', error.message);
      
      // If login failed, try to check if the user exists
      try {
        console.log('\nChecking if user exists...');
        const query = new Parse.Query(Parse.User);
        query.equalTo('username', TEST_USERNAME);
        const existingUser = await query.first({ useMasterKey: true });
        
        if (existingUser) {
          console.log(`User ${TEST_USERNAME} exists in the database.`);
        } else {
          console.log(`User ${TEST_USERNAME} does not exist in the database.`);
        }
      } catch (userCheckError) {
        console.error('Failed to check if user exists:', userCheckError.message);
      }
    }
    
    // Test 3: Create a test object
    try {
      console.log('\nTest 3: Creating a test object...');
      
      const TestObject = Parse.Object.extend('TestObject');
      const testObject = new TestObject();
      testObject.set('foo', 'bar');
      testObject.set('timestamp', new Date());
      
      const savedObject = await testObject.save(null, { useMasterKey: true });
      console.log(`✅ Test object created with ID: ${savedObject.id}`);
    } catch (error) {
      console.error('❌ Failed to create test object:', error.message);
    }
    
  } catch (error) {
    console.error('Fatal error during tests:', error);
  }
}

// Run the tests
testParseConnection().then(() => {
  console.log('\nTests completed.');
});