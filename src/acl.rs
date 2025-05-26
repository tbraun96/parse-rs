// src/acl.rs
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Represents a Parse Access Control List (ACL).
///
/// ACLs are used to control permissions for reading and writing ParseObjects.
/// Permissions can be granted to the public, individual users (by user ID),
/// or roles.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseACL {
    permissions: HashMap<String, AccessLevel>,
}

/// Defines the access level for a user or role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AccessLevel {
    #[serde(skip_serializing_if = "Option::is_none")]
    read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    write: Option<bool>,
}

impl ParseACL {
    /// Creates a new, empty `ParseACL`.
    /// By default, no one has any permissions.
    pub fn new() -> Self {
        ParseACL {
            permissions: HashMap::new(),
        }
    }

    /// Sets public read access.
    ///
    /// # Arguments
    /// * `allowed`: `true` to allow public read access, `false` to disallow.
    pub fn set_public_read_access(&mut self, allowed: bool) {
        self.permissions
            .entry("*".to_string())
            .or_insert_with(|| AccessLevel {
                read: None,
                write: None,
            })
            .read = Some(allowed);
    }

    /// Sets public write access.
    ///
    /// # Arguments
    /// * `allowed`: `true` to allow public write access, `false` to disallow.
    pub fn set_public_write_access(&mut self, allowed: bool) {
        self.permissions
            .entry("*".to_string())
            .or_insert_with(|| AccessLevel {
                read: None,
                write: None,
            })
            .write = Some(allowed);
    }

    /// Sets read access for a specific user ID.
    ///
    /// # Arguments
    /// * `user_id`: The object ID of the user.
    /// * `allowed`: `true` to allow read access for this user, `false` to disallow.
    pub fn set_user_read_access(&mut self, user_id: &str, allowed: bool) {
        self.permissions
            .entry(user_id.to_string())
            .or_insert_with(|| AccessLevel {
                read: None,
                write: None,
            })
            .read = Some(allowed);
    }

    /// Sets write access for a specific user ID.
    ///
    /// # Arguments
    /// * `user_id`: The object ID of the user.
    /// * `allowed`: `true` to allow write access for this user, `false` to disallow.
    pub fn set_user_write_access(&mut self, user_id: &str, allowed: bool) {
        self.permissions
            .entry(user_id.to_string())
            .or_insert_with(|| AccessLevel {
                read: None,
                write: None,
            })
            .write = Some(allowed);
    }

    /// Sets read access for a specific role.
    ///
    /// # Arguments
    /// * `role_name`: The name of the role (e.g., "Administrators").
    /// * `allowed`: `true` to allow read access for this role, `false` to disallow.
    pub fn set_role_read_access(&mut self, role_name: &str, allowed: bool) {
        let role_key = format!("role:{}", role_name);
        self.permissions
            .entry(role_key)
            .or_insert_with(|| AccessLevel {
                read: None,
                write: None,
            })
            .read = Some(allowed);
    }

    /// Sets write access for a specific role.
    ///
    /// # Arguments
    /// * `role_name`: The name of the role (e.g., "Administrators").
    /// * `allowed`: `true` to allow write access for this role, `false` to disallow.
    pub fn set_role_write_access(&mut self, role_name: &str, allowed: bool) {
        let role_key = format!("role:{}", role_name);
        self.permissions
            .entry(role_key)
            .or_insert_with(|| AccessLevel {
                read: None,
                write: None,
            })
            .write = Some(allowed);
    }

    // --- Getter methods ---

    /// Gets whether the public is allowed to read this object.
    pub fn get_public_read_access(&self) -> bool {
        self.permissions
            .get("*")
            .and_then(|access| access.read)
            .unwrap_or(false)
    }

    /// Gets whether the public is allowed to write this object.
    pub fn get_public_write_access(&self) -> bool {
        self.permissions
            .get("*")
            .and_then(|access| access.write)
            .unwrap_or(false)
    }

    /// Gets whether the given user is allowed to read this object.
    pub fn get_user_read_access(&self, user_id: &str) -> bool {
        self.permissions
            .get(user_id)
            .and_then(|access| access.read)
            .unwrap_or(false)
    }

    /// Gets whether the given user is allowed to write this object.
    pub fn get_user_write_access(&self, user_id: &str) -> bool {
        self.permissions
            .get(user_id)
            .and_then(|access| access.write)
            .unwrap_or(false)
    }

    /// Gets whether users belonging to the given role are allowed to read this object.
    pub fn get_role_read_access(&self, role_name: &str) -> bool {
        let role_key = format!("role:{}", role_name);
        self.permissions
            .get(&role_key)
            .and_then(|access| access.read)
            .unwrap_or(false)
    }

    /// Gets whether users belonging to the given role are allowed to write this object.
    pub fn get_role_write_access(&self, role_name: &str) -> bool {
        let role_key = format!("role:{}", role_name);
        self.permissions
            .get(&role_key)
            .and_then(|al| al.write)
            .unwrap_or(false)
    }
}

impl Default for ParseACL {
    fn default() -> Self {
        Self::new()
    }
}

// Custom serialization for ParseACL to match the Parse Server format
impl Serialize for ParseACL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Clean up permissions before serializing to remove entries that are effectively false
        // However, Parse Server expects explicit false if one permission is set and the other is not.
        // For simplicity, we'll serialize what's there. The server handles defaults.
        // A more robust cleanup might be needed if specific server behavior is targeted.
        serializer.collect_map(self.permissions.iter().filter_map(|(k, v)| {
            // Only include if there's at least one explicit permission
            if v.read.is_some() || v.write.is_some() {
                Some((k, v))
            } else {
                None
            }
        }))
    }
}

// Custom deserialization for ParseACL
impl<'de> Deserialize<'de> for ParseACL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, AccessLevel>::deserialize(deserializer)?;
        Ok(ParseACL { permissions: map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_acl_new() {
        let acl = ParseACL::new();
        assert!(acl.permissions.is_empty());
    }

    #[test]
    fn test_set_public_read_access() {
        let mut acl = ParseACL::new();
        acl.set_public_read_access(true);
        assert_eq!(acl.permissions.get("*").unwrap().read, Some(true));
        assert_eq!(acl.permissions.get("*").unwrap().write, None);

        acl.set_public_read_access(false);
        assert_eq!(acl.permissions.get("*").unwrap().read, Some(false));
    }

    #[test]
    fn test_set_public_write_access() {
        let mut acl = ParseACL::new();
        acl.set_public_write_access(true);
        assert_eq!(acl.permissions.get("*").unwrap().write, Some(true));
        assert_eq!(acl.permissions.get("*").unwrap().read, None);

        acl.set_public_write_access(false);
        assert_eq!(acl.permissions.get("*").unwrap().write, Some(false));
    }

    #[test]
    fn test_set_user_read_access() {
        let mut acl = ParseACL::new();
        acl.set_user_read_access("userId123", true);
        assert_eq!(acl.permissions.get("userId123").unwrap().read, Some(true));
    }

    #[test]
    fn test_set_user_write_access() {
        let mut acl = ParseACL::new();
        acl.set_user_write_access("userId123", true);
        assert_eq!(acl.permissions.get("userId123").unwrap().write, Some(true));
    }

    #[test]
    fn test_set_role_read_access() {
        let mut acl = ParseACL::new();
        acl.set_role_read_access("Admin", true);
        assert_eq!(acl.permissions.get("role:Admin").unwrap().read, Some(true));
    }

    #[test]
    fn test_set_role_write_access() {
        let mut acl = ParseACL::new();
        acl.set_role_write_access("Editor", true);
        assert_eq!(
            acl.permissions.get("role:Editor").unwrap().write,
            Some(true)
        );
    }

    #[test]
    fn test_acl_serialization_empty() {
        let acl = ParseACL::new();
        let json_string = serde_json::to_string(&acl).unwrap();
        assert_eq!(json_string, "{}");
    }

    #[test]
    fn test_acl_serialization_public_read() {
        let mut acl = ParseACL::new();
        acl.set_public_read_access(true);
        let json_string = serde_json::to_string(&acl).unwrap();
        assert_eq!(json_string, "{\"*\":{\"read\":true}}");
    }

    #[test]
    fn test_acl_serialization_public_write() {
        let mut acl = ParseACL::new();
        acl.set_public_write_access(true);
        let json_string = serde_json::to_string(&acl).unwrap();
        assert_eq!(json_string, "{\"*\":{\"write\":true}}");
    }

    #[test]
    fn test_acl_serialization_public_read_write() {
        let mut acl = ParseACL::new();
        acl.set_public_read_access(true);
        acl.set_public_write_access(false);
        let json_string = serde_json::to_string(&acl).unwrap();
        assert_eq!(json_string, "{\"*\":{\"read\":true,\"write\":false}}");
    }

    #[test]
    fn test_acl_serialization_user_and_role() {
        let mut acl = ParseACL::new();
        acl.set_user_read_access("user1", true);
        acl.set_role_write_access("Admin", true);
        let json_string = serde_json::to_string(&acl).unwrap();
        // HashMap order is not guaranteed, so check for content
        assert!(json_string.contains("\"user1\":{\"read\":true}"));
        assert!(json_string.contains("\"role:Admin\":{\"write\":true}"));
        assert!(json_string.starts_with("{") && json_string.ends_with("}"));
    }

    #[test]
    fn test_acl_deserialization_empty() {
        let json_string = "{}";
        let acl: ParseACL = serde_json::from_str(json_string).unwrap();
        assert!(acl.permissions.is_empty());
    }

    #[test]
    fn test_acl_deserialization_public_read() {
        let json_string = "{\"*\":{\"read\":true}}";
        let acl: ParseACL = serde_json::from_str(json_string).unwrap();
        assert_eq!(acl.permissions.get("*").unwrap().read, Some(true));
        assert_eq!(acl.permissions.get("*").unwrap().write, None);
    }

    #[test]
    fn test_acl_deserialization_user_and_role() {
        let json_string = "{\"user1\":{\"read\":true},\"role:Admin\":{\"write\":true}}";
        let acl: ParseACL = serde_json::from_str(json_string).unwrap();
        assert_eq!(acl.permissions.get("user1").unwrap().read, Some(true));
        assert_eq!(acl.permissions.get("role:Admin").unwrap().write, Some(true));
    }

    #[test]
    fn test_acl_default() {
        let acl: ParseACL = Default::default();
        assert!(acl.permissions.is_empty());
    }
}
