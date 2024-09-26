use std::{fmt::Debug, path::PathBuf};

/// Common capabilities supported by [`Document`](Document)s, [`Folder`](Folder)s and [`PathBuf`](std::path::PathBuf)s
///
/// Note: this trait is object-safe, which means it can be used as variable, function parameter and function return
/// types.
///
/// ```
/// fn test2() {
///     let a: &[&dyn FileSystemEntity] = &[
///         &Document::at(User(Pictures([])), "pic", Create::No),
///         &User(Pictures([])),
///         &Project(Data([]).with_id("qualifier", "organization", "application")),
///         &PathBuf::new(),
///     ];
///     for b in a {
///         println!("{b:?} {} exist.", if b.exists() {"does"} else {"doesn't"});
///     }
/// }
/// ```
pub trait FileSystemEntity: Debug {
    /// The full path of this FileSystemEntity. Returns an empty String if the file path could not be accessed.
    ///
    /// DANGER: the format of file paths is different between systems.
    fn path(&self) -> String;
    /// The name (with extension if applicable) of this FileSystemEntity. Returns an empty String if the path could not be accessed.
    fn name(&self) -> String;
    /// Whether this FileSystemEntity exists.
    fn exists(&self) -> bool;
}

impl FileSystemEntity for PathBuf {
    fn name(&self) -> String {
        self.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
    fn path(&self) -> String {
        self.display().to_string()
    }
    fn exists(&self) -> bool {
        match self.try_exists() {
            Ok(value) if value => true,
            _ => false,
        }
    }
}
