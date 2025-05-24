pub mod client;
pub mod cloud;
pub mod error;
pub mod object;
pub mod query;
pub mod types;
pub mod user;

pub use client::ParseClient as Parse; // Alias for convenience
pub use error::ParseError;
pub use object::ParseObject;
pub use query::ParseQuery;
pub use user::ParseUser;

// Re-export key types from the types module if needed directly
pub use types::ParseDate;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
