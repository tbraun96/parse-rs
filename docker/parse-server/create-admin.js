// create-admin.js
const Parse = require('parse/node');

const appId = process.env.PARSE_SERVER_APPLICATION_ID; // Align with compose env var name
const masterKey = process.env.PARSE_SERVER_MASTER_KEY; // Align with compose env var name
const PARSE_PORT = process.env.PARSE_SERVER_PORT;
if (!PARSE_PORT) {
    console.error('Error: PARSE_SERVER_PORT is not set');
    process.exit(1);
}
const serverURL = `http://0.0.0.0:${PARSE_PORT}/parse`; // Hardcode for internal container use
const adminUsername = process.env.ADMIN_USERNAME;
const adminPassword = process.env.ADMIN_PASSWORD;

if (!appId || !masterKey || !serverURL || !adminUsername || !adminPassword) {
  console.error('Error: Missing required environment variables for admin creation script.');
  process.exit(1);
}

console.log(`Initializing Parse SDK for server: ${serverURL}`);
Parse.initialize(appId, null, masterKey); // Use null for JS key when using Master Key
Parse.serverURL = serverURL;

async function createAdminUser() {
  try {
    console.log(`Attempting to create admin user: ${adminUsername}`);

    // Check if user already exists
    const userQuery = new Parse.Query(Parse.User);
    userQuery.equalTo('username', adminUsername);
    const existingUser = await userQuery.first({ useMasterKey: true });

    if (existingUser) {
      console.log(`User ${adminUsername} already exists. Updating password...`);
      // Reset password for the existing user
      existingUser.setPassword(adminPassword);
      await existingUser.save(null, { useMasterKey: true });
      console.log('Existing user password updated successfully.');
      return; // Exit after updating password
    }

    // Create the user
    const user = new Parse.User();
    user.set('username', adminUsername);
    user.set('email', adminUsername); // Often username is email for admin
    user.set('password', adminPassword);
    user.set('emailVerified', true); // Pre-verify admin email

    await user.signUp(null, { useMasterKey: true });
    console.log(`Admin user ${adminUsername} created successfully with ID: ${user.id}`);

    // Optional: Add user to Admin Role
    const roleQuery = new Parse.Query(Parse.Role);
    roleQuery.equalTo('name', 'Admin');
    let adminRole = await roleQuery.first({ useMasterKey: true });

    if (!adminRole) {
      console.log('Admin role not found, creating it...');
      const acl = new Parse.ACL();
      acl.setPublicReadAccess(true); // Or restrict as needed
      adminRole = new Parse.Role('Admin', acl);
      await adminRole.save(null, { useMasterKey: true });
      console.log('Admin role created.');
    }

    adminRole.getUsers().add(user);
    await adminRole.save(null, { useMasterKey: true });
    console.log(`User ${adminUsername} added to Admin role.`);

  } catch (error) {
    console.error('Error creating admin user:', error);
    process.exit(1);
  }
}

createAdminUser();
