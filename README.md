# documents
Documents is an ergonomic, intuitive and beginner-friendly library for dealing with files and folders.

## Examples

```rust
use documents::prelude::*;

fn main() {
    test1();
    test2();
}

fn test1() {
    with(
        [
            Document::at(User(Pictures([])), "1.png", Create::No),
            Document::at(User(Documents([])), "README.txt", Create::AutoRenameIfExists),
            Document::at(
                User(Pictures(["Movie Trailer"])),
                "thumbnail.png",
                Create::No,
            )
            .alias("pic"),
            Document::at(User(Downloads([])), "file.txt", Create::No),
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

fn test2() {
    let a: &[&dyn FileSystemEntity] = &[
        &Document::at(User(Pictures([])), "pic", Create::No),
        &User(Pictures([])),
        &Project(Data([]).with_id("qualifier", "organization", "application")),
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
```