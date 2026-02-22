//! Simple traits for adding context to and printing results/errors.

#[cfg(feature = "log")]
extern crate log;

#[cfg(feature = "log")]
use log::Level;

use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::path::Path;



/// An error with added context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualError<E> {
  pub error: E,
  pub context: String
}

impl<E> ContextualError<E> {
  pub fn new(error: E, context: String) -> Self {
    ContextualError { error, context }
  }
}

impl<E> Display for ContextualError<E>
where E: Display {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} ({})", self.context, self.error)
  }
}

impl<E> Error for ContextualError<E>
where E: Error + 'static {
  #[inline]
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Option::Some(&self.error)
  }
}

impl<E> From<(E, String)> for ContextualError<E> {
  fn from((error, context): (E, String)) -> Self {
    ContextualError { error, context }
  }
}



/// An extension trait for [`Result<T, E>`][Result] that allows
/// them to be easily converted to `Result<T, ContextualError<E>>`
pub trait Contextualize {
  type Output;

  fn context(self, message: impl Into<String>) -> Self::Output;

  #[doc(hidden)]
  fn context_with(self, message_provider: impl FnOnce() -> String) -> Self::Output where Self: Sized {
    self.context((message_provider)())
  }

  fn context_path(self, message: impl Display, path: impl AsRef<Path>) -> Self::Output where Self: Sized {
    self.context_with(|| format!("{} in {}", message, path.as_ref().display()))
  }
}

impl<T, E> Contextualize for Result<T, E> {
  type Output = Result<T, ContextualError<E>>;

  fn context(self, message: impl Into<String>) -> Self::Output {
    self.map_err(|error| ContextualError { error, context: message.into() })
  }

  fn context_with(self, message_provider: impl FnOnce() -> String) -> Self::Output {
    self.map_err(|error| ContextualError { error, context: (message_provider)() })
  }
}

macro_rules! maybe {
  ($value:expr, $error:pat, $out:expr) => {
    match $value {
      Ok(v) => Some(v),
      Err($error) => {
        $out;
        None
      }
    }
  };
}

/// An extension trait for [`Result<T, E>`][Result] that allows
/// the error variant to be split off and printed with [`println!`].
pub trait Print {
  type Output;

  #[track_caller]
  fn print(self) -> Self::Output;
}

impl<T, E> Print for Result<T, E>
where E: Display {
  type Output = Option<T>;

  #[track_caller]
  #[inline]
  fn print(self) -> Self::Output {
    maybe!(self, error, println!("{error}"))
  }
}

/// An extension trait for [`Result<T, E>`][Result] that allows
/// the error variant to be split off and sent to any of the macros provided by the [`log`] crate.
#[cfg(feature = "log")]
pub trait Log: Print {
  #[track_caller]
  fn log(self, target: &str, level: Level) -> Self::Output;
  #[track_caller]
  fn log_error(self) -> Self::Output;
  #[track_caller]
  fn log_warn(self) -> Self::Output;
  #[track_caller]
  fn log_info(self) -> Self::Output;
  #[track_caller]
  fn log_debug(self) -> Self::Output;
  #[track_caller]
  fn log_trace(self) -> Self::Output;
}

#[cfg(feature = "log")]
impl<T, E> Log for Result<T, E>
where E: Display {
  #[track_caller]
  #[inline]
  fn log(self, target: &str, level: Level) -> Self::Output {
    maybe!(self, message, log::log!(target: target, level, "{message}"))
  }

  #[track_caller]
  #[inline]
  fn log_error(self) -> Self::Output {
    maybe!(self, message, log::error!("{message}"))
  }

  #[track_caller]
  #[inline]
  fn log_warn(self) -> Self::Output {
    maybe!(self, message, log::warn!("{message}"))
  }

  #[track_caller]
  #[inline]
  fn log_info(self) -> Self::Output {
    maybe!(self, message, log::info!("{message}"))
  }

  #[track_caller]
  #[inline]
  fn log_debug(self) -> Self::Output {
    maybe!(self, message, log::debug!("{message}"))
  }

  #[track_caller]
  #[inline]
  fn log_trace(self) -> Self::Output {
    maybe!(self, message, log::trace!("{message}"))
  }
}
