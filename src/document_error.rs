use std::{error::Error, fmt::Display};

/// This library's error types.
///
/// Note: functions in this library will not actually return this concrete error type.
/// Instead a Box<dyn Error> will be returned. Print it to the console to see a description of the error.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DocumentError {
    /// "User directories not found"
    UserDirsNotFound,
    /// "Pictures directory not found"
    PicturesDirNotFound,
    /// "Videos directory not found"
    VideosDirNotFound,
    /// "Downloads directory not found"
    DownloadsDirNotFound,
    /// "Documents directory not found"
    DocumentsDirNotFound,
    /// "Project directories not found"
    ProjectDirsNotFound,
    /// "File not found: (file path)"
    FileNotFound(String),
    /// "Could not create file: (file path)"
    CouldNotCreateFile(String),
    /// "Could not create parent folder: (parent directory path)"
    CouldNotCreateParentFolder(String),
    /// "Could not launch file with default app: (file path)"
    CouldNotLaunchFile(String),
    /// "Could not open file: (file path)"
    CouldNotOpenFile(String),
    /// "File not writable: (file path)"
    FileNotWritable(String),
    /// "File not open: (file path)"
    FileNotOpen(String),
}

impl Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg: String = match self {
            Self::UserDirsNotFound => "User directories not found".to_string(),
            Self::PicturesDirNotFound => "Pictures directory not found".to_string(),
            Self::VideosDirNotFound => "Videos directory not found".to_string(),
            Self::DownloadsDirNotFound => "Downloads directory not found".to_string(),
            Self::FileNotFound(file_path) => "File not found: ".to_string() + file_path,
            Self::CouldNotCreateFile(file_path) => {
                "Could not create file: ".to_string() + file_path
            }
            Self::CouldNotCreateParentFolder(parent_folder_path) => {
                "Could not create parent folder: ".to_string() + parent_folder_path
            }
            Self::CouldNotLaunchFile(file_path) => {
                "Could not launch file with default app: ".to_string() + file_path
            }
            Self::ProjectDirsNotFound => "Project directories not found".to_string(),
            Self::CouldNotOpenFile(file_path) => "Could not open file: ".to_string() + file_path,
            Self::DocumentsDirNotFound => "Documents directory not found".to_string(),
            Self::FileNotWritable(file_path) => "File not writable: ".to_string() + file_path,
            Self::FileNotOpen(file_path) => "File not open: ".to_string() + file_path,
        };
        f.pad(msg.as_str())
    }
}

impl Error for DocumentError {
    fn description(&self) -> &str {
        "Document error"
    }
}
