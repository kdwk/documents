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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
