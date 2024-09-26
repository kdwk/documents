use core::fmt::Debug;
use core::str;
use extend::ext;
use open;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Lines, Read, Write};
use std::path::{Path, PathBuf};

use crate::{Create, DocumentError, FileSystemEntity, Folder, Mode};

/// A type that represents a file.
///
/// Create an instance of this type with [`Document::at`](Document::at) or [`Document::at_path`](Document::at_path).
/// Do not use direct instantiation.
///
/// Optionally, use [`with`](with) to create Documents for use within a scope --- it's easier!
///
/// Note: a Document is not the actual file. Creating an instance of this type will not create a new file.
/// To specify whether to do so, use the `create` parameter of [`Document::at`](Document::at) or [`Document::at_path`](Document::at_path).
#[derive(Clone, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Document {
    /// The alias of this Document in a [`DocumentMap`](DocumentMap), used to retrieve this Document from the DocumentMap.
    pub(crate) alias: String,
    /// The [`PathBuf`](std::path::PathBuf) of this Document. You can use `.display()` to convert it to something printable.
    pub(crate) pathbuf: PathBuf,
    /// The [`Create`](Create) policy of this Document, used to signal whether a new file should be created when creating an instance of Document.
    /// Can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`
    pub(crate) create_policy: Create,
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.name(), self.path())
    }
}

fn parse_filepath(pathbuf: PathBuf) -> (String, Option<i64>, Option<String>) {
    let mut name = pathbuf.name();
    let extension = match ".".to_string()
        + pathbuf
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    {
        extension if extension == "." => None,
        extension => Some(extension),
    };
    if let Some(extension) = &extension {
        name = match name.clone().strip_suffix(extension.as_str()) {
            Some(new_name) => new_name.to_string(),
            None => name,
        };
    }
    let open_bracket_index = (&name).rfind("(");
    let close_bracket_index = (&name).rfind(")");
    let mut duplicate_number = None;
    if let Some(open_bracket_index) = open_bracket_index {
        if let Some(close_bracket_index) = close_bracket_index {
            duplicate_number = match name
                .split_at(open_bracket_index)
                .1
                .split_at(close_bracket_index - open_bracket_index)
                .0
                .parse()
            {
                Ok(number) => {
                    name = match name.strip_suffix(format!("({})", number).as_str()) {
                        Some(new_name) => new_name.to_string(),
                        None => name,
                    };
                    Some(number)
                }
                Err(_) => None,
            }
        }
    }
    (name, duplicate_number, extension)
}

impl Document {
    fn setup(
        mut pathbuf: PathBuf,
        create: Create,
        dry_run: bool,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let (name, duplicate_number_option, extension_option) = parse_filepath(pathbuf.clone());
        let mut duplicate_number = 0;
        let mut extension = String::new();
        if let Some(number) = duplicate_number_option {
            duplicate_number = number;
        }
        if let Some(ext) = extension_option {
            extension = ext;
        }
        match create {
            Create::OnlyIfNotExists => {
                if let Some(parent_folder) = pathbuf.clone().parent() {
                    if let Err(_) = create_dir_all(parent_folder) {
                        Err(DocumentError::CouldNotCreateParentFolder(
                            parent_folder.to_path_buf().display().to_string(),
                        ))?
                    }
                }
                if !pathbuf.exists() && !dry_run {
                    OpenOptions::new()
                        .read(false)
                        .write(true)
                        .create_new(true)
                        .open(pathbuf.clone())?;
                }
            }
            Create::AutoRenameIfExists => {
                if let Some(parent_folder) = pathbuf.clone().parent() {
                    if let Err(_) = create_dir_all(parent_folder) {
                        Err(DocumentError::CouldNotCreateParentFolder(
                            parent_folder.to_path_buf().display().to_string(),
                        ))?
                    }
                }
                while pathbuf.exists() {
                    duplicate_number += 1;
                    let new_filename = name.clone()
                        + "("
                        + duplicate_number.to_string().as_str()
                        + ")"
                        + if extension.clone().len() > 0 && extension.clone() != "." {
                            extension.as_str()
                        } else {
                            ""
                        };
                    pathbuf = pathbuf
                        .clone()
                        .parent()
                        .unwrap_or(Path::new(""))
                        .join(new_filename);
                }
                if !dry_run {
                    OpenOptions::new()
                        .read(false)
                        .write(true)
                        .create_new(true)
                        .open(pathbuf.clone())?;
                }
            }
            _ => {}
        }
        if !pathbuf.exists() && !dry_run {
            Err(DocumentError::FileNotFound(pathbuf.path()))?
        }
        Ok(pathbuf)
    }

    /// Create an instance of [`Document`](Document) from a [`Folder`](Folder) location.
    ///
    /// *location*: the [`Folder`](Folder) which the file is in, e.g. `User(Pictures(["Screenshots"]))` or
    /// `Project(Data([])).with_id("com", "github.kdwk", "Spidey")`.
    ///
    /// *filename*: the name of the file with its file extension. Provide anything that can be converted to a string:
    /// a [`String`](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    /// A [`PathBuf`](std::path::PathBuf) can also be converted to an acceptable type with `.display()`.
    ///
    /// *create*: the [`Create`](Create) policy of this Document, i.e. whether this operation will create a new file.
    /// This can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`.
    ///
    /// The `filename` will be used as the [`alias`](Document::alias) of this Document. Change it with `.alias()`.
    ///
    /// If the file does not exist, or if the create policy cannot be carried out, this function will return an error.
    pub fn at<const N: usize>(
        location: Folder<N>,
        filename: impl Display,
        create: Create,
    ) -> Result<Self, Box<dyn Error>> {
        let mut pathbuf = location.into_pathbuf_result(filename.to_string())?;
        let original_name = pathbuf.name();
        pathbuf = Document::setup(pathbuf, create, false)?;
        Ok(Self {
            alias: original_name,
            pathbuf,
            create_policy: create,
        })
    }

    /// Create an instance of [`Document`](Document) from the full file path.
    ///
    /// DANGER: the format of file paths and specific location of files can differ between computers!
    /// Always prefer to put files in well-known folders like the Downloads folder or your project's data folder --- use [Document::at](Document::at) for that.
    /// Use this function only if you are very confident the path is valid, such as if other libraries provide file paths for you to use.
    ///
    /// *path*: the full file path of the file. Provide anything that can be converted to a string: a [`String`](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    /// A [`PathBuf`](std::path::PathBuf) can also be converted to an acceptable type with `display()`.
    ///
    /// *alias*: the alias used to retrieve this Document from a [`DocumentMap`](DocumentMap). Provide anything that can be converted to a string: a [`String`](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    ///
    /// *create*: the [`Create`](Create) policy of this Document, i.e. whether this operation will create a new file. This can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`.
    ///
    /// If the file does not exist, or if the create policy cannot be carried out, this function will return an error.
    pub fn at_path(
        path: impl Display,
        alias: impl Display,
        create: Create,
    ) -> Result<Self, Box<dyn Error>> {
        let mut pathbuf = PathBuf::from(path.to_string());
        pathbuf = Document::setup(pathbuf, create, false)?;
        Ok(Self {
            alias: alias.to_string(),
            pathbuf,
            create_policy: create,
        })
    }
    fn open_file(&self, permissions: Mode) -> Result<File, Box<dyn Error>> {
        match OpenOptions::new()
            .read(permissions.readable())
            .write(permissions.writable())
            .append(permissions.appendable())
            .open(self.pathbuf.clone())
        {
            Ok(file) => Ok(file),
            Err(_) => Err(DocumentError::CouldNotOpenFile(self.path()))?,
        }
    }

    /// Launch the file with the default app. Equivalent to opening the file from a file manager.
    ///
    /// Returns an error if the file could not be launched.
    pub fn launch_with_default_app(&self) -> Result<&Self, Box<dyn Error>> {
        if let Err(_) = open::that_detached(self.path()) {
            Err(DocumentError::CouldNotLaunchFile(self.path()))?
        } else {
            Ok(self)
        }
    }

    /// Convert this Document to a [`File`](std::fs::File) (the standard library's type to represent a file). Useful if other functions or libraries expect a File,
    /// or if you need to perform operations on the file not supported by this Document.
    ///
    /// *permissions*: the [`Mode`](Mode) with which the file will be opened, can be `Mode::Read`, `Mode::Replace`, `Mode::Append`, `Mode::ReadReplace` and `Mode::ReadAppend`.
    ///
    /// Returns an error if the file cannot be opened.
    pub fn file(&mut self, permissions: Mode) -> Result<File, Box<dyn Error>> {
        self.open_file(permissions)
    }

    /// Add content to the end of the file represented by this Document.
    ///
    /// *content*: bytes to be appended. If you have a string literal add `b` to convert it to bytes (`b"example"`); if you have an `&str` or `String` convert with `.as_bytes()`.
    /// If other libraries provide you with bytes, e.g. from a download operation, plug it in as-is.
    ///
    /// Returns an error if the file cannot be opened or the write operation fails.
    pub fn append(&mut self, content: &[u8]) -> Result<&mut Self, Box<dyn Error>> {
        let mut file = self.open_file(Mode::Append)?;
        file.write_all(content)?;
        Ok(self)
    }

    /// Replace the contents of the file represented by this Document.
    ///
    /// DANGER: irreversibly wipes out the entire file before writing new content.
    ///
    /// *content*: bytes to overwrite with. If you have a string literal add `b` to convert it to bytes (`b"example"`); if you have an `&str` or `String` convert with `.as_bytes()`.
    /// If other libraries provide you with bytes, e.g. from a download operation, you can plug it in as-is.
    ///
    /// Returns an error if the file cannot be opened or the write operation fails.
    pub fn replace_with(&mut self, content: &[u8]) -> Result<&mut Self, Box<dyn Error>> {
        let mut file = self.open_file(Mode::Replace)?;
        file.write_all(content)?;
        Ok(self)
    }

    /// Returns an iterator over the lines of the file represented by this Document.
    ///
    /// Returns an error if the file could not be opened in read mode.
    ///
    /// Useful for processing the contents of file line by line.
    ///
    /// ```
    /// for line in document.lines().expect("Could not read lines")  {
    ///     println!("{line}");
    /// }
    /// ```
    pub fn lines(&self) -> Result<Lines<BufReader<File>>, Box<dyn Error>> {
        let file = self.open_file(Mode::Read)?;
        Ok(BufReader::new(file).lines())
    }

    /// Returns the contents of the file represented by this Document.
    ///
    /// Returns an error if the file could not be opened in read mode or
    /// its content could not be written to a String.
    ///
    /// ```
    /// let file_content = document.content().expect("Could not read content");
    /// println!("{file_content}");
    /// ```
    pub fn content(&self) -> Result<String, Box<dyn Error>> {
        let mut file = self.open_file(Mode::Read)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Ok(string)
    }

    /// The file extension of the file represented by this Document.
    ///
    /// Returns an empty String if the file extension is empty or could
    /// not be converted to a String
    pub fn extension(&self) -> String {
        self.pathbuf
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
}

#[ext(pub)]
impl Result<Document, Box<dyn Error>> {
    /// Sets the alias of this Document if this Document has created successfully. Use "_" to instruct [`with(...)`](with) to skip adding this Document to the [`DocumentMap`](DocumentMap).
    ///
    /// Note: the alias is used to identify this Document in a [DocumentMap](DocumentMap).
    /// Do not provide the same alias for multiple Documents.
    ///
    /// Returns an error if this Document has not been created successfully.
    fn alias(self, alias: &str) -> Result<Document, Box<dyn Error>> {
        match self {
            Ok(mut document) => {
                document.alias = String::from(alias);
                Ok(document)
            }
            Err(error) => Err(error),
        }
    }

    /// Suggest a rename of this Document if there is already a file at that path.
    ///
    /// e.g. if `picture.png` already exists, this will return `picture(1).png`. If `picture(1).png` already exists, this will return `picture(2).png`, etc.
    ///
    /// Returns the same path if there is no existing file at that path.
    ///
    /// Returns an empty String if this Document has not been created successfully.
    fn suggest_rename(&self) -> String {
        match self {
            Ok(document) => {
                match Document::setup(document.pathbuf.clone(), Create::AutoRenameIfExists, true) {
                    Ok(new_name) => new_name.path(),
                    Err(error) => {
                        eprintln!("{}", error);
                        "".to_string()
                    }
                }
            }
            Err(error) => match error.downcast_ref::<DocumentError>() {
                Some(document_error) => match document_error {
                    DocumentError::FileNotFound(path) => path.clone(),
                    _ => "".to_string(),
                },
                None => "".to_string(),
            },
        }
    }
}

#[ext(pub)]
impl Lines<BufReader<File>> {
    /// Print out this file line by line.
    ///
    /// Returns an error if the line cannot be read.
    fn print(self) -> Result<(), Box<dyn Error>> {
        for line in self {
            println!("{}", line?);
        }
        Ok(())
    }
}

impl FileSystemEntity for Document {
    fn name(&self) -> String {
        self.pathbuf
            .clone()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
    fn path(&self) -> String {
        self.pathbuf.display().to_string()
    }
    fn exists(&self) -> bool {
        self.pathbuf.exists()
    }
}

impl FileSystemEntity for Result<Document, Box<dyn Error>> {
    fn exists(&self) -> bool {
        match self {
            Ok(document) => document.exists(),
            Err(_) => false,
        }
    }
    fn name(&self) -> String {
        match self {
            Ok(document) => document.name(),
            Err(_) => "".to_string(),
        }
    }
    fn path(&self) -> String {
        match self {
            Ok(document) => document.path(),
            Err(_) => "".to_string(),
        }
    }
}
