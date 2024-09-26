use serde::{Deserialize, Serialize};

/// Whether to create a new file to be represented by this Document.
///
/// *No*: do not create a new file under any circumstances. If the file does not exist, the [`Document`](Document) instance will fail to be created.
///
/// *OnlyIfNotExists*: create a new file if the file does not exist.
///
/// *AutoRenameIfExists*: create a new file under all circumstances. If a file of the same name already exists in the specified folder,
/// add (1), (2), etc. to the file name to avoid collision (before the file extension).
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum Create {
    #[default]
    No,
    OnlyIfNotExists,
    AutoRenameIfExists,
}
