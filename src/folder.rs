use std::{
    fmt::Display,
    path::{Path, PathBuf},
    usize,
};

use crate::{DocumentError, FileSystemEntity};

/// A type that represents well-known folders that are likely to exist on most devices.
///
/// See also [`User`](User) and [`Project`](Project)
///
/// e.g.
///
/// `User(Pictures([]))`: the user's Pictures folder
///
/// `Project(Data(["Ad Filters"]).with_id("com", "github.kdwk", "Spidey"))`: subfolder "Ad Filters" under the application's data folder, with app ID com.github.kdwk.Spidey (see [Project](Project))
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Folder<'a, const N: usize> {
    User(User<'a, N>),
    Project((Project<'a, N>, &'a str, &'a str, &'a str)),
}

fn join_all(path: &Path, subdirs: &[&str]) -> PathBuf {
    let mut pathbuf = path.to_path_buf();
    for subdir in subdirs {
        pathbuf.push(subdir);
    }
    pathbuf
}

impl<'a, const N: usize> Folder<'a, N> {
    pub(crate) fn into_pathbuf_result(
        &self,
        filename: impl Display,
    ) -> Result<PathBuf, DocumentError> {
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
///```
/// pub enum User<'a, const N: usize> {
///     Documents([&'a str; N]),
///     Pictures([&'a str; N]),
///     Videos([&'a str; N]),
///     Downloads([&'a str; N]),
///     Home([&'a str; N]),
/// }
/// ```
///
/// Put subdirectories under the respective folders like so: `Pictures(["Screenshots", "July", "14"])`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum User<'a, const N: usize> {
    Documents([&'a str; N]),
    Pictures([&'a str; N]),
    Videos([&'a str; N]),
    Downloads([&'a str; N]),
    Home([&'a str; N]),
}

/// A type that represents the application's project folder. An isolated folder is usually provided per app per user by the operating system for apps to put internal files.
///
/// DANGER: if your software is not a registered app on the operating system, this folder may not exist.
/// In this case, consider using a custom subfolder under a [`User`](User) folder instead.
///
/// Note: use this type with [`.with_id(...)`](Project::with_id) to let [`with`](with) get the folder which is assigned to your app by the operating system.
///
/// Put subdirectories under the respective folders like so: `Data(["Ad Filters", "English"])`
///
/// *Config*: place configuration files here, such as app settings.
///
/// *Data*: place data files here, such as a web browser's adblock filters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Project<'a, const N: usize> {
    Config([&'a str; N]),
    Data([&'a str; N]),
}

impl<'a, const N: usize> Project<'a, N> {
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

impl<'a, const N: usize> FileSystemEntity for Folder<'a, N> {
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
            .display()
            .to_string()
    }
}
