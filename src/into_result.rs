use std::{error::Error, fmt::Display};

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
