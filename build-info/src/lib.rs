extern crate build_info_macros;

#[cfg(feature = "chrono")]
pub extern crate chrono;

#[cfg(feature = "git")]
pub use build_info_macros::{
  git_hash,
  git_hash_short,
  git_branch,
  git_root
};

#[cfg(feature = "chrono")]
pub use build_info_macros::{
  build_datetime_utc,
  build_datetime_utc_format,
  build_datetime_utc_rfc2822,
  build_datetime_utc_rfc3339,
  build_datetime_local_fixed,
  build_datetime_local_fixed_format,
  build_datetime_local_fixed_rfc2822,
  build_datetime_local_fixed_rfc3339,
  build_datetime_naive,
  build_datetime_naive_format,
  build_date_naive,
  build_date_naive_format,
  build_time_naive,
  build_time_naive_format
};
