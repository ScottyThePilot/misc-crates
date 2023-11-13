//! At the center of the cosmos (the application) is the great tree Yggdrasil (the executable).
//! Often, we may find that an unlucky wanderer (the user) has strayed away from its safety
//! (opened the application from outside of its directory). It is in these moments that we ought
//! to take it upon ourselves to return them (the current root directory) to the great tree
//! (the executable's directory) with [`reroot`], lest they forget what is most important to them
//! (we can't access files relative to the executable correctly).

use std::io;
use std::env;
use std::path::PathBuf;

/// Gets the current application root (directory of the executable).
pub fn root() -> io::Result<PathBuf> {
  if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
    return Ok(PathBuf::from(manifest_dir));
  };

  // Though foolish the wanderer may be, no foolishness surpasses that of Microsoft Windows.
  let mut current_exe = dunce::canonicalize(env::current_exe()?)?;

  if current_exe.pop() {
    return Ok(current_exe);
  };

  Err(io::Error::new(
    io::ErrorKind::Other,
    "failed to find an application root"
  ))
}

/// Set the current root directory to the application root (directory of the executable).
pub fn reroot() -> io::Result<()> {
  root().and_then(env::set_current_dir)
}
