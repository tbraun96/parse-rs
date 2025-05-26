use serde::{Deserialize, Serialize};

/// Represents a file field as stored in a ParseObject.
/// This struct is used for serialization and deserialization
/// when a ParseFile is part of a ParseObject.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileField {
    pub name: String,
    pub url: String,
    #[serde(rename = "__type")]
    pub _type: String, // Should always be "File"
}

impl FileField {
    // Constructor for internal use, ensuring _type is always "File"
    pub fn new(name: String, url: String) -> Self {
        FileField {
            name,
            url,
            _type: "File".to_string(),
        }
    }
}

/// Represents a file to be uploaded to Parse Server or a file
/// that has already been uploaded.
#[derive(Debug, Clone)]
pub struct ParseFile {
    /// The name of the file. This could be the original filename
    /// or the name assigned by Parse Server upon upload.
    pub name: String,
    /// The URL of the file after it has been uploaded to Parse Server.
    /// This will be `None` for a file that hasn't been uploaded yet.
    pub url: Option<String>,
    /// The MIME type of the file (e.g., "image/jpeg", "text/plain").
    pub mime_type: String,
    /// The raw byte data of the file.
    pub data: Vec<u8>,
}

impl ParseFile {
    /// Creates a new `ParseFile` instance with the given name, data, and MIME type.
    /// The URL will be `None` initially and should be populated after uploading.
    pub fn new(name: String, data: Vec<u8>, mime_type: String) -> Self {
        ParseFile {
            name,
            url: None,
            mime_type,
            data,
        }
    }

    /// Converts this `ParseFile` into a `FileField` suitable for embedding
    /// within a `ParseObject`. Returns `None` if the file has not been uploaded
    /// (i.e., if `url` is `None`).
    pub fn to_field(&self) -> Option<FileField> {
        self.url.as_ref().map(|u| FileField {
            name: self.name.clone(),
            url: u.clone(),
            _type: "File".to_string(),
        })
    }
}
