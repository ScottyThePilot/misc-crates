extern crate build_info;

#[allow(unused)]
#[cfg(feature = "git")]
mod git_example {
  pub const GIT_HASH: &str = build_info::git_hash!();
  pub const GIT_HASH_SHORT: &str = build_info::git_hash_short!();
  pub const GIT_BRANCH: &str = build_info::git_branch!();
  pub const GIT_ROOT: &str = build_info::git_root!();
}

#[allow(unused)]
#[cfg(feature = "chrono")]
mod chrono_example {
  extern crate chrono;

  use chrono::{DateTime, Utc, FixedOffset};

  pub const BUILD_DATETIME_UTC: DateTime<Utc> = build_info::build_datetime_utc!();
  pub const BUILD_DATETIME_UTC_FORMAT: &str = build_info::build_datetime_utc_format!("%d/%m/%Y %H:%M:%S %Z");
  pub const BUILD_DATETIME_UTC_RFC2822: &str = build_info::build_datetime_utc_rfc2822!();
  pub const BUILD_DATETIME_UTC_RFC3339: &str = build_info::build_datetime_utc_rfc3339!();
  pub const BUILD_DATETIME_LOCAL_FIXED: DateTime<FixedOffset> = build_info::build_datetime_local_fixed!();
  pub const BUILD_DATETIME_LOCAL_FIXED_FORMAT: &str = build_info::build_datetime_local_fixed_format!("%d/%m/%Y %H:%M:%S %Z");
  pub const BUILD_DATETIME_LOCAL_FIXED_RFC2822: &str = build_info::build_datetime_local_fixed_rfc2822!();
  pub const BUILD_DATETIME_LOCAL_FIXED_RFC3339: &str = build_info::build_datetime_local_fixed_rfc3339!();
}

#[allow(unused)]
#[cfg(all(feature = "chrono", feature = "git"))]
mod git_chrono_example {
  extern crate chrono;

  use chrono::{DateTime, Utc, FixedOffset};

  pub const GIT_LAST_COMMIT_DATETIME_UTC: DateTime<Utc> = build_info::git_last_commit_datetime_utc!();
  pub const GIT_LAST_COMMIT_DATETIME_UTC_FORMAT: &str = build_info::git_last_commit_datetime_utc_format!("%d/%m/%Y %H:%M:%S %Z");
  pub const GIT_LAST_COMMIT_DATETIME_UTC_RFC2822: &str = build_info::git_last_commit_datetime_utc_rfc2822!();
  pub const GIT_LAST_COMMIT_DATETIME_UTC_RFC3339: &str = build_info::git_last_commit_datetime_utc_rfc3339!();
  pub const GIT_LAST_COMMIT_DATETIME_LOCAL_FIXED: DateTime<FixedOffset> = build_info::git_last_commit_datetime_local_fixed!();
  pub const GIT_LAST_COMMIT_DATETIME_LOCAL_FIXED_FORMAT: &str = build_info::git_last_commit_datetime_local_fixed_format!("%d/%m/%Y %H:%M:%S %Z");
  pub const GIT_LAST_COMMIT_DATETIME_LOCAL_FIXED_RFC2822: &str = build_info::git_last_commit_datetime_local_fixed_rfc2822!();
  pub const GIT_LAST_COMMIT_DATETIME_LOCAL_FIXED_RFC3339: &str = build_info::git_last_commit_datetime_local_fixed_rfc3339!();
}

fn main() {}
