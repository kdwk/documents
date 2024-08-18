use std::collections::HashMap;
use std::error::Error;

mod mode;
use mode::Mode;
mod folder;
use folder::Folder;
mod create;
use create::Create;
mod document_error;
use document_error::DocumentError;
mod document;
use document::Document;
mod filesystem_entity;
use filesystem_entity::FileSystemEntity;
mod document_map;
use document_map::DocumentMap;
mod into_result;
use into_result::IntoResult;

/// A way to declare all of the [`Document`](Document)s in one place then access them in the `closure` through a [`DocumentMap`](DocumentMap) by their [`alias`](Document::alias)es.
///
/// *documents*: a [`slice`](https://doc.rust-lang.org/std/primitive.slice.html) of Result of [`Document`](Document)s,
/// which are usually provided by [`Document::at()`](Document::at) or [`Document::at_path()`](Document::at_path).
///
/// *closure*: a [closure](https://doc.rust-lang.org/book/ch13-01-closures.html) which accepts a [`DocumentMap`](DocumentMap) as parameter, and can use [`Document`](Document)s in its body.
/// This function will run this closure with a [`DocumentMap`](DocumentMap) of [`Document`](Document)s provided in `documents`.
/// This closure should return a type that implements [`IntoResult`](IntoResult) any of: [`()`](https://doc.rust-lang.org/std/primitive.unit.html),
/// [`Option<T>`](std::option::Option) or [`Result<(), Box<dyn Error>>`](std::result::Result).
/// Therefore, `?` (try) operators can be used on `Result`s and `Option`s in this closure as long as all of the `?`s are used on the same type.
///
/// Note: if any of the [`Document`](Document)s fail to be created, i.e. returns an error, the `closure` will NOT be run.
/// Errors encountered during Document setup or returned from the closure will be printed.
///
/// Note: to conduct write operations, including `.append(...)` and `.replace(...)` on [`Document`](Document)s, declare the [`DocumentMap`](DocumentMap) parameter of *closure* to be mutable.
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
    Closure: FnOnce(DocumentMap) -> Return,
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
    match closure(DocumentMap(document_map)).into_result() {
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
    pub use crate::{
        create::Create,
        document::{Document, LinesBufReaderFileExt, ResultDocumentBoxErrorExt},
        filesystem_entity::FileSystemEntity,
        folder::{
            Folder::{self, Project, User},
            Project::{Config, Data},
            User::{Documents, Downloads, Home, Pictures, Videos},
        },
        mode::Mode,
        with,
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
                for (alias, doc) in d.clone() {
                    println!("{alias}: {doc:?}");
                }
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
            println!(
                "{:?} {} exist.",
                b,
                if b.exists() { "does" } else { "doesn't" }
            );
        }
    }
}
