use core::fmt::Debug;
use directories;
use extend::ext;
use open;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Lines, Write};
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};

/// The mode of opening the file. Describes what you are permitted to do with it.
///
/// *Read*: permission to read the file only.
///
/// *Replace*: permission wipe the file and replace its contents.
///
/// *Append*: permission to add content to the end of the file.
///
/// *ReadReplace*: permission to read the file, and also to wipe the file and replace its contents.
///
/// *ReadAppend*: permission to read the file, and also to add content to the end of the file.
///
/// Read, ReadReplace, ReadAppend are `read`-able.
///
/// Replace, Append, ReadReplace, ReadAppend are `write`-able.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Mode {
    #[default]
    Read,
    Replace,
    Append,
    ReadReplace,
    ReadAppend,
}

impl Mode {
    /// ```
    /// match self {
    ///     Self::Read | Self::ReadReplace | Self::ReadAppend => true,
    ///     _ => false,
    /// }
    /// ```
    pub fn readable(&self) -> bool {
        match self {
            Self::Read | Self::ReadReplace | Self::ReadAppend => true,
            _ => false,
        }
    }

    /// ```
    /// match self {
    ///     Self::Replace | Self::Append | Self::ReadAppend | Self::ReadReplace => true,
    ///     _ => false,
    /// }
    /// ```
    pub fn writable(&self) -> bool {
        match self {
            Self::Replace | Self::Append | Self::ReadAppend | Self::ReadReplace => true,
            _ => false,
        }
    }

    /// ```
    /// match self {
    ///     Self::Append | Self::ReadAppend => true,
    ///     _ => false,
    /// }
    /// ```
    pub fn appendable(&self) -> bool {
        match self {
            Self::Append | Self::ReadAppend => true,
            _ => false,
        }
    }
}

/// A type that represents well-known folders that are likely to exist on most devices.
///
/// See also [`User`](User) and [`Project`](Project)
///
/// e.g.
///
/// `User(Pictures(&[]))`: the user's Pictures folder
///
/// `Project(Data(&["Ad Filters"]).with_id("com", "github.kdwk", "Spidey"))`: subfolder "Ad Filters" under the application's data folder, with app ID com.github.kdwk.Spidey (see [Project](Project))
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Folder<'a> {
    User(User<'a>),
    Project((Project<'a>, &'a str, &'a str, &'a str)),
}

fn join_all(path: &Path, subdirs: &[&str]) -> PathBuf {
    let mut pathbuf = path.to_path_buf();
    for subdir in subdirs {
        pathbuf.push(subdir);
    }
    pathbuf
}

impl<'a> Folder<'a> {
    fn into_pathbuf_result(&self, filename: impl Display) -> Result<PathBuf, DocumentError> {
        match self {
            Folder::User(subdir) => match subdir {
                User::Pictures(subdirs) => {
                    if let Some(dir) = directories::UserDirs::new() {
                        if let Some(path) = dir.picture_dir() {
                            let mut pathbuf = join_all(path, subdirs);
                            pathbuf = pathbuf.join(filename.to_string());
                            Ok(pathbuf)
                        } else {
                            Err(DocumentError::PicturesDirNotFound)?
                        }
                    } else {
                        Err(DocumentError::UserDirsNotFound)?
                    }
                }
                User::Videos(subdirs) => {
                    if let Some(dir) = directories::UserDirs::new() {
                        if let Some(path) = dir.video_dir() {
                            let mut pathbuf = join_all(path, subdirs);
                            pathbuf = pathbuf.join(filename.to_string());
                            Ok(pathbuf)
                        } else {
                            Err(DocumentError::VideosDirNotFound)?
                        }
                    } else {
                        Err(DocumentError::UserDirsNotFound)?
                    }
                }
                User::Downloads(subdirs) => {
                    if let Some(dir) = directories::UserDirs::new() {
                        if let Some(path) = dir.download_dir() {
                            let mut pathbuf = join_all(path, subdirs);
                            pathbuf = pathbuf.join(filename.to_string());
                            Ok(pathbuf)
                        } else {
                            Err(DocumentError::DownloadsDirNotFound)?
                        }
                    } else {
                        Err(DocumentError::UserDirsNotFound)?
                    }
                }
                User::Documents(subdirs) => {
                    if let Some(dir) = directories::UserDirs::new() {
                        if let Some(path) = dir.document_dir() {
                            let mut pathbuf = join_all(path, subdirs);
                            pathbuf = pathbuf.join(filename.to_string());
                            Ok(pathbuf)
                        } else {
                            Err(DocumentError::DocumentsDirNotFound)?
                        }
                    } else {
                        Err(DocumentError::UserDirsNotFound)?
                    }
                }
                User::Home(subdirs) => {
                    if let Some(dir) = directories::UserDirs::new() {
                        let path = dir.home_dir();
                        let mut pathbuf = join_all(path, subdirs);
                        pathbuf = pathbuf.join(filename.to_string());
                        Ok(pathbuf)
                    } else {
                        Err(DocumentError::UserDirsNotFound)?
                    }
                }
            },
            Folder::Project((subdir, qualifier, organization, application)) => match subdir {
                Project::Data(subdirs) => {
                    if let Some(dir) =
                        directories::ProjectDirs::from(qualifier, organization, application)
                    {
                        let mut pathbuf = join_all(dir.data_dir(), subdirs);
                        pathbuf = pathbuf.join(filename.to_string());
                        Ok(pathbuf)
                    } else {
                        Err(DocumentError::ProjectDirsNotFound)?
                    }
                }
                Project::Config(subdirs) => {
                    if let Some(dir) =
                        directories::ProjectDirs::from(qualifier, organization, application)
                    {
                        let mut pathbuf = join_all(dir.config_dir(), subdirs);
                        pathbuf = pathbuf.join(filename.to_string());
                        Ok(pathbuf)
                    } else {
                        Err(DocumentError::ProjectDirsNotFound)?
                    }
                }
            },
        }
    }
}

/// A type that represents well-known user folders.
///
/// Put subdirectories in the [`slice`]() like so: `Pictures(&["Screenshots", "July", "14"])`.
/// 
/// e.g.
/// 
/// 
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum User<'a> {
    Documents(&'a [&'a str]),
    Pictures(&'a [&'a str]),
    Videos(&'a [&'a str]),
    Downloads(&'a [&'a str]),
    Home(&'a [&'a str]),
}

/// A type that represents the application's project folder. An isolated folder is usually provided per app per user by the operating system for apps to put internal files.
///
/// DANGER: if your software is not a registered app on the operating system, this folder may not exist.
/// In this case, consider using a custom subfolder under a [`User`](User) folder instead.
///
/// Note: use this type with [`.with_id(...)`](Project::with_id) to let [`with`](with) get the folder which is assigned to your app by the operating system.
///
/// Put subdirectories under the respective folders like so: `Data(&["Ad Filters", "English"])`
///
/// *Config*: place configuration files here, such as app settings.
///
/// *Data*: place data files here, such as a web browser's adblock filters.
/// 
/// ```
/// let adblock_filters_folder = Project(Data(&["Ad Filters", "English"]));
/// let 
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Project<'a> {
    Config(&'a [&'a str]),
    Data(&'a [&'a str]),
}

impl<'a> Project<'a> {
    /// The app ID should have the reverse-DNS format of "com.example.App", where "com" is the qualifier, "example" is the organization and "App" is the app's name.
    ///
    /// Note: this app ID should be the same app ID you provide to the operating system to uniquely identify your app.
    /// Windows calls this ID the Application User Model ID, Apple platforms call this the Bundle ID, Android and Linux call this the App ID.
    pub fn with_id(
        self,
        qualifier: &'a str,
        organization: &'a str,
        application: &'a str,
    ) -> (Self, &'a str, &'a str, &'a str) {
        (self, qualifier, organization, application)
    }
}

/// Whether to create a new file to be represented by this Document.
///
/// *No*: do not create a new file under any circumstances. If the file does not exist, the [`Document`](Document) instance will fail to be created.
///
/// *OnlyIfNotExists*: create a new file if the file does not exist.
///
/// *AutoRenameIfExists*: create a new file under all circumstances. If a file of the same name already exists in the specified folder,
/// add (1), (2), etc. to the file name to avoid collision (before the file extension).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Create {
    #[default]
    No,
    OnlyIfNotExists,
    AutoRenameIfExists,
}

/// This library's error types.
///
/// Note: functions in this library will not actually return this concrete error type.
/// Instead a Box<dyn Error> will be returned. Print it to the console to see a description of the error.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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

/// A type that represents a file.
///
/// Create an instance of this type with [`Document::at`](Document::at) or [`Document::at_path`](Document::at_path).
/// Do not use direct instantiation.
///
/// Optionally, use [`with`](with) to create Documents for use within a scope --- it's easier!
///
/// Note: a Document is not the actual file. Creating an instance of this type will not create a new file.
/// To specify whether to do so, use the `create` parameter of [`Document::at`](Document::at) or [`Document::at_path`](Document::at_path).
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Document {
    /// The alias of this Document in a [`Map`](Map), used to retrieve this Document from the Map.
    alias: String,
    /// The [`PathBuf`](std::path::PathBuf) of this Document. You can use `.display()` to convert it to something printable.
    pathbuf: PathBuf,
    /// The [`Create`](Create) policy of this Document, used to signal whether a new file should be created when creating an instance of Document.
    /// Can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`
    create_policy: Create,
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
    /// *location*: the [`Folder`](Folder) which the file is in, e.g. `User(Pictures(&["Screenshots"]))` or
    /// `Project(Data(&[])).with_id("com", "github.kdwk", "Spidey")`.
    ///
    /// *filename*: the name of the file with its file extension. Provide anything that can be converted to a string:
    /// a [`String`](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    /// A [`PathBuf`](std::path::PathBuf) can also be converted to an acceptable type with `display()`.
    ///
    /// *create*: the [`Create`](Create) policy of this Document, i.e. whether this operation will create a new file.
    /// This can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`.
    ///
    /// The `filename` will be used as the [`alias`](Document::alias) of this Document. Change it with `.alias()`.
    ///
    /// If the file does not exist, or if the create policy cannot be carried out, this function will return an error.
    pub fn at(
        location: Folder,
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
    /// *alias*: the alias used to retrieve this Document from a [`Map`](Map). Provide anything that can be converted to a string: a [`String`](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
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

    /// The file extension of the file represented by this Document.
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
    /// Sets the alias of this Document if this Document has created successfully. Use "_" to instruct [`with(...)`](with) to skip adding this Document to the [Map](Map).
    ///
    /// Note: the alias is used to identify this Document in a [Map](Map).
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

/// Common capabilities supported by [`Document`](Document)s, [`Folder`](Folder)s and [`PathBuf`](std::path::PathBuf)s
/// 
/// Note: this trait is object-safe, which means it can be used as the variable, function parameter and function return
/// types.
/// 
/// ```
/// fn test2() {
///     let a: &[&dyn FileSystemEntity] = &[
///         &Document::at(User(Pictures(&[""])), "pic", Create::No),
///         &User(Pictures(&[""])),
///         &Project(Data(&[]).with_id("qualifier", "organization", "application")),
///         &PathBuf::new(),
///     ];
///     for b in a {
///         println!("{:?} {} exist.", b, if b.exists() {"does"} else {"doesn't"});
///     }
/// }
/// ```
pub trait FileSystemEntity: Debug {
    /// The full path of this FileSystemEntity. Returns an empty String if the file path could not be accessed.
    ///
    /// DANGER: the format of file paths is different between systems.
    fn path(&self) -> String;
    /// The full path of this FileSystemEntity. Returns an empty String if the file path could not be accessed.
    fn name(&self) -> String;
    /// Whether this FileSystemEntity exists.
    fn exists(&self) -> bool;
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
        self.pathbuf
            .as_os_str()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
    fn exists(&self) -> bool {
        self.pathbuf.exists()
    }
}

impl<'a> FileSystemEntity for Folder<'a> {
    fn exists(&self) -> bool {
        self.into_pathbuf_result("").unwrap_or_default().exists()
    }
    fn name(&self) -> String {
        self.into_pathbuf_result("")
            .unwrap_or_default()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
    fn path(&self) -> String {
        self.into_pathbuf_result("")
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
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
        self.to_str().unwrap_or_default().to_string()
    }
    fn exists(&self) -> bool {
        match self.try_exists() {
            Ok(value) if value => true,
            _ => false,
        }
    }
}

impl FileSystemEntity for Result<Document, Box<dyn Error>> {
    fn exists(&self) -> bool {
        match self {
            Ok(document) => document.exists(),
            Err(_) => false
        }
    }
    fn name(&self) -> String {
        match self {
            Ok(document) => document.name(),
            Err(_) => "".to_string()
        }
    }
    fn path(&self) -> String {
        match self {
            Ok(document) => document.path(),
            Err(_) => "".to_string()
        }
    }
}

/// A type that wraps a HashMap between a String and Documents. Access the Documents with any type of index that can be converted to a String.
///
/// An instance of this type is provided by [`with`](with) containing all of the [`Document`](Document)s
/// given in the `documents` parameter as the values, and their respective [`alias`](Document::alias)es as keys.
pub struct Map(HashMap<String, Document>);

impl<'a, Str> Index<Str> for Map
where
    Str: Display,
{
    type Output = Document;
    fn index(&self, index: Str) -> &Self::Output {
        &self.0[index.to_string().as_str()]
    }
}

impl<'a, Str> IndexMut<Str> for Map
where
    Str: Display,
{
    fn index_mut(&mut self, index: Str) -> &mut Self::Output {
        self.0.get_mut(index.to_string().as_str()).unwrap()
    }
}

/// Maps types implementing this trait to `Result<(), Box<dyn Error>>`.
/// 
/// Note: this trait is *not* object-safe, which means it cannot be used as the type of a variable. 
/// However, `impl IntoResult` can be used for function parameters and return types.
/// 
/// ```
/// fn may_fail_on_paper() -> impl IntoResult {
///     doesnt_actually_fail(); 
///     // Returns (), acceptable
/// }
/// 
/// fn may_fail_for_real() -> impl IntoResult {
///     let value: T = get_value_or_fail()?;
///     use(value);
///     Ok(()) // Returns Result<(), Error>, acceptable 
/// }
/// 
/// fn may_be_none() -> impl IntoResult {
///     let value: T = get_value_or_none()?;
///     use(value);
///     Some(()) // Returns Option<()>, acceptable 
/// }
/// ```
///
/// Implemented for [`()`](https://doc.rust-lang.org/std/primitive.unit.html),
/// [`Option<T>`](std::option::Option) and [`Result<(), Box<dyn Error>>`](std::result::Result) out-of-the-box
pub trait IntoResult {
    fn into_result(self) -> Result<(), Box<dyn Error>>;
}

impl IntoResult for () {
    /// Implementation
    /// 
    /// ```
    /// fn into_result(self) -> Result<(), Box<dyn Error>> {
    ///     Ok(())
    /// }
    /// ```
    fn into_result(self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl<T> IntoResult for Option<T> {
    /// Implementation
    /// 
    /// ```
    /// fn into_result(self) -> Result<(), Box<dyn Error>> {
    ///     match self {
    ///         Some(_) => Ok(()),
    ///         None => Err(Box::new(NoneError)),
    ///     }
    /// }
    /// ```
    fn into_result(self) -> Result<(), Box<dyn Error>> {
        match self {
            Some(_) => Ok(()),
            None => Err(Box::new(NoneError)),
        }
    }
}

impl<T> IntoResult for Result<T, Box<dyn Error>> {
    /// Implementation
    /// 
    /// ```
    /// fn into_result(self) -> Result<(), Box<dyn Error>> {
    ///     match self {
    ///         Ok(_) => Ok(()),
    ///         Err(error) => Err(error),
    ///     }
    /// }
    /// ```
    fn into_result(self) -> Result<(), Box<dyn Error>> {
        match self {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }
}

/// NoneError: Expected Some(...), got None.
#[derive(Debug, Clone, Copy)]
struct NoneError;

impl Error for NoneError {
    /// "NoneError: Expected Some(...), got None."
    fn description(&self) -> &str {
        "NoneError: Expected Some(...), got None."
    }
}

impl Display for NoneError {
    /// "NoneError: expected Some(...), got None."
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("NoneError: expected Some(...), got None.")
    }
}

/// A way to declare all of the [`Document`](Document)s in one place then access them in the `closure` through a [`Map`](Map) by their [`alias`](Document::alias)es.
///
/// *documents*: a [`slice`](https://doc.rust-lang.org/std/primitive.slice.html) of Result of [`Document`](Document)s,
/// which are usually provided by [`Document::at()`](Document::at) or [`Document::at_path()`](Document::at_path).
///
/// *closure*: a [closure](https://doc.rust-lang.org/book/ch13-01-closures.html) which accepts a [`Map`](Map) as parameter, and can use [`Document`](Document)s in its body.
/// This function will run this closure with a [`Map`](Map) of [`Document`](Document)s provided in `documents`.
/// This closure should return a type that implements [`IntoResult`](IntoResult) any of: [`()`](https://doc.rust-lang.org/std/primitive.unit.html),
/// [`Option<T>`](std::option::Option) or [`Result<(), Box<dyn Error>>`](std::result::Result).
/// Therefore, `?` (try) operators can be used on `Result`s and `Option`s in this closure as long as all of the `?`s are used on the same type.
///
/// Note: if any of the [`Document`](Document)s fail to be created, i.e. returns an error, the `closure` will NOT be run.
/// Errors encountered during Document setup or returned from the closure will be printed.
/// 
/// Note: to conduct write operations, including `.append(...)` and `.replace(...)` on [`Document`](Document)s, declare the [`Map`](Map) parameter of *closure* to be mutable.
///
/// e.g.
/// ```
/// with(
///     &[
///         Document::at(User(Pictures(&[])), "1.png", Create::No),
///         Document::at(
///             User(Pictures(&["Movie Trailer"])),
///             "thumbnail.png",
///             Create::OnlyIfNotExists,
///         )
///         .alias("pic"),
///         Document::at(User(Downloads(&[])), "file.txt", Create::AutoRenameIfExists),
///     ],
///     |mut d| {
///         println!("{}", d["1.png"].name());
///         d["pic"].launch_with_default_app()?;
///         d["file.txt"]
///             .append(b"Something\nto be added")?
///             .launch_with_default_app()?
///             .lines()?
///             .print()?;
///         Ok(())
///     },
/// );
pub fn with<Closure, Return>(documents: &[Result<Document, Box<dyn Error>>], closure: Closure)
where
    Closure: FnOnce(Map) -> Return,
    Return: IntoResult,
{
    let mut document_map = HashMap::new();
    for document_result in documents {
        let document = match document_result {
            Ok(document) => (*document).clone(),
            Err(error) => {
                eprintln!("{}", error);
                return;
            }
        };
        let document_alias = document.alias.clone();
        if document_alias != "_" {
            document_map.insert(document_alias, document);
        }
    }
    match closure(Map(document_map)).into_result() {
        Ok(_) => {}
        Err(error) => eprintln!("{error}"),
    }
}

/// A convenient way to import all useful structs, traits and functions in this library.
///
/// Note: remember to add the `documents` crate to the project Cargo.toml first.
///
/// ```
/// use documents::prelude::*;
/// ```
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::{
        with, Create, Document, FileSystemEntity,
        Folder::{self, Project, User},
        IntoResult, LinesBufReaderFileExt, Mode,
        Project::{Config, Data},
        ResultDocumentBoxErrorExt,
        User::{Documents, Downloads, Home, Pictures, Videos},
    };
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::prelude::*;
    #[test]
    /// This test doesn't do anything yet.
    fn test1() {
        with(
            &[
                Document::at(User(Pictures(&[])), "1.png", Create::No),
                Document::at(User(Pictures(&[])), "42-44.png", Create::No),
                Document::at(
                    User(Pictures(&["Movie Trailer"])),
                    "thumbnail.png",
                    Create::No,
                )
                .alias("pic"),
                Document::at(User(Downloads(&[])), "file.txt", Create::No),
            ],
            |mut d| {
                println!("{}", d["1.png"].name());
                d["pic"].launch_with_default_app()?;
                d["file.txt"]
                    .append(b"Something\nto be added")?
                    .launch_with_default_app()?
                    .lines()?
                    .print()?;
                Ok(())
            },
        );
    }
    #[test]
    /// This test also doesn't do anything yet.
    fn test2() {
        let a: &[&dyn FileSystemEntity] = &[
            &Document::at(User(Pictures(&[""])), "pic", Create::No),
            &User(Pictures(&[""])),
            &Project(Data(&[]).with_id("qualifier", "organization", "application")),
            &PathBuf::new(),
        ];
        for b in a {
            println!("{:?} {} exist.", b, if b.exists() {"does"} else {"doesn't"});
        }
    }
}
