use directories;
use extend::ext;
use open;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Lines, Write};
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};

/// The mode of opening the file. Describes what you are permitted to do with it.
///
/// Read: permission to read the file only.
///
/// Replace: permission wipe the file and replace its contents.
///
/// Append: permission to add content to the end of the file.
///
/// ReadReplace: permission to read the file, and also to wipe the file and replace its contents.
///
/// ReadAppend: permission to read the file, and also to add content to the end of the file.
///
/// Read, ReadReplace, ReadAppend are `read`-able.
///
/// Replace, Append, ReadReplace, ReadAppend are `write`-able.
#[derive(Debug, Clone, Copy)]
pub enum Mode {
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
///
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

pub enum User<'a> {
    Documents(&'a [&'a str]),
    Pictures(&'a [&'a str]),
    Videos(&'a [&'a str]),
    Downloads(&'a [&'a str]),
    Home(&'a [&'a str]),
}

pub enum Project<'a> {
    Config(&'a [&'a str]),
    Data(&'a [&'a str]),
}

impl<'a> Project<'a> {
    /// The app ID should have the reverse-DNS format of "com.example.App", where "com" is the qualifier, "example" is the organization and "App" is the application
    pub fn with_id(
        self,
        qualifier: &'a str,
        organization: &'a str,
        application: &'a str,
    ) -> (Self, &'a str, &'a str, &'a str) {
        (self, qualifier, organization, application)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Create {
    No,
    OnlyIfNotExists,
    AutoRenameIfExists,
}

#[derive(Debug)]
pub enum DocumentError {
    UserDirsNotFound,
    PicturesDirNotFound,
    VideosDirNotFound,
    DownloadsDirNotFound,
    DocumentsDirNotFound,
    ProjectDirsNotFound,
    FileNotFound(String),
    CouldNotCreateFile(String),
    CouldNotCreateParentFolder(String),
    CouldNotLaunchFile(String),
    CouldNotOpenFile(String),
    FileNotWritable(String),
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
/// Create an instance of this type with [Document::at](Document::at) or [Document::from_path](Document::from_path).
/// Do not use direct instantiation.
///
/// Optionally, use [with](with) to create Documents for use within a scope --- it's easier!
///
/// Note that a Document is not the actual file. Creating an instance of this type will not create a new file.
/// To specify whether to do so, use the `create` parameter of [Document::at](Document::at) or [Document::from_path](Document::from_path).
#[derive(Debug, Clone)]
pub struct Document {
    /// The alias of this Document in a [Map](Map), used to retrieve this Document from the Map.
    alias: String,
    /// The [PathBuf](PathBuf) of this Document. You can use [.display()](Pathbuf::display) to convert it to something printable.
    pathbuf: PathBuf,
    /// The [create policy](Create) of this Document, used to signal whether a new file should be created when creating an instance of Document.
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
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
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
                            parent_folder
                                .to_path_buf()
                                .to_str()
                                .unwrap_or("")
                                .to_string(),
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
                            parent_folder
                                .to_path_buf()
                                .to_str()
                                .unwrap_or("")
                                .to_string(),
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
                        .unwrap_or(&Path::new(""))
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

    /// Create an instance of [Document](Document) from a [Folder](Folder) location.
    ///
    /// location: the [Folder](Folder) which the file is in, e.g. `User(Pictures(&["Screenshots"]))` or `Project(Data(&[])).with_id("com", "github.kdwk", "Spidey")`.
    ///
    /// filename: the name of the file with its file extension. Provide anything that can be converted to a string: a [String](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    ///
    /// create: the [Create](Create) policy of this Document, i.e. whether this operation will create a new file. This can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`.
    ///
    /// The file name will be used as the [alias](Document::alias) of this Document. Change with [.alias()](Document::alias).
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

    /// Create an instance of [Document](Document) from the full file path.
    ///
    /// DANGER: the format of file paths and specific location of files can differ between computers!
    /// Always prefer to put files in well-known folders like the Downloads folder or your project's data folder --- use [Document::at](Document::at) for that.
    /// Use this function only if you are very confident the path is valid, such as if other libraries provide file paths for you to use.
    ///
    /// path: the full file path of the file. Provide anything that can be converted to a string: a [String](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    ///
    /// alias: the alias used to retrieve this Document from a [Map](Map). Provide anything that can be converted to a string: a [String](std::string::String) (`String::new("example")`) or &str (`"example"`) --- anything goes.
    ///
    /// create: the [Create](Create) policy of this Document, i.e. whether this operation will create a new file. This can be `Create::No`, `Create::OnlyIfNotExists` or `Create::AutoRenameIfExists`.
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

    /// Convert this Document to a [File](std::fs::File) (the standard library's type to represent a file). Useful if other functions or libraries expect a File,
    /// or if you need to perform operations on the file not supported by this Document.
    ///
    /// permissions: the [Mode](Mode) with which the file will be opened, can be `Mode::Read`, `Mode::Replace`, `Mode::Append`, `Mode::ReadReplace` and `Mode::ReadAppend`.
    ///
    /// Returns an error if the file cannot be opened.
    pub fn file(&mut self, permissions: Mode) -> Result<File, Box<dyn Error>> {
        self.open_file(permissions)
    }

    /// Add content to the end of the file represented by this Document.
    ///
    /// content: bytes to be appended. If you have a string literal add `b` to convert it to bytes (`b"example"`); if you have an `&str` or `String` convert with `.as_bytes()`.
    /// If other libraries provide you with bytes, e.g. from a download operation, you can plug it in as-is.
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
    /// content: bytes to overwrite with. If you have a string literal add `b` to convert it to bytes (`b"example"`); if you have an `&str` or `String` convert with `.as_bytes()`.
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
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_string()
    }
}

#[ext(pub)]
impl Result<Document, Box<dyn Error>> {
    fn alias(self, alias: &str) -> Result<Document, Box<dyn Error>> {
        match self {
            Ok(mut document) => {
                document.alias = String::from(alias);
                Ok(document)
            }
            Err(error) => Err(error),
        }
    }
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
    fn print(self) -> Result<(), Box<dyn Error>> {
        for line in self {
            println!("{}", line?);
        }
        Ok(())
    }
}

pub trait FileSystemEntity {
    fn path(&self) -> String;
    fn name(&self) -> String;
    fn exists(&self) -> bool;
}

impl FileSystemEntity for Document {
    fn name(&self) -> String {
        self.pathbuf
            .clone()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("")
            .to_string()
    }
    fn path(&self) -> String {
        self.pathbuf.as_os_str().to_str().unwrap_or("").to_string()
    }
    fn exists(&self) -> bool {
        self.pathbuf.exists()
    }
}

impl<'a> FileSystemEntity for Folder<'a> {
    fn exists(&self) -> bool {
        self.into_pathbuf_result("")
            .unwrap_or(PathBuf::new())
            .exists()
    }
    fn name(&self) -> String {
        self.into_pathbuf_result("")
            .unwrap_or(PathBuf::new())
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_string()
    }
    fn path(&self) -> String {
        self.into_pathbuf_result("")
            .unwrap_or(PathBuf::new())
            .to_str()
            .unwrap_or("")
            .to_string()
    }
}

impl FileSystemEntity for PathBuf {
    fn name(&self) -> String {
        self.file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_string()
    }
    fn path(&self) -> String {
        self.to_str().unwrap_or("").to_string()
    }
    fn exists(&self) -> bool {
        match self.try_exists() {
            Ok(value) if value => true,
            _ => false,
        }
    }
}

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

pub trait IntoResult {
    fn into_result(self) -> Result<(), Box<dyn Error>>;
}

impl IntoResult for () {
    fn into_result(self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl<T> IntoResult for Result<T, Box<dyn Error>> {
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

impl<T> IntoResult for Option<T> {
    fn into_result(self) -> Result<(), Box<dyn Error>> {
        match self {
            Some(_) => Ok(()),
            None => Err(NoneError)?,
        }
    }
}

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
        if document.clone().alias != "_" {
            document_map.insert(document.clone().alias, document);
        }
    }
    match closure(Map(document_map)).into_result() {
        Ok(_) => {}
        Err(error) => eprintln!("{error}"),
    }
}
