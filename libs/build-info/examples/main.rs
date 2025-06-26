extern crate build_info;
extern crate chrono;

use chrono::{DateTime, NaiveDateTime, NaiveDate, NaiveTime, Utc, FixedOffset};

pub const GIT_HASH: &str = build_info::git_hash!();
pub const GIT_HASH_SHORT: &str = build_info::git_hash_short!();
pub const GIT_BRANCH: &str = build_info::git_branch!();
pub const GIT_ROOT: &str = build_info::git_root!();
pub const BUILD_DATETIME_UTC: DateTime<Utc> = build_info::build_datetime_utc!();
pub const BUILD_DATETIME_UTC_FORMAT: &str = build_info::build_datetime_utc_format!("%d/%m/%Y %H:%M:%S %Z");
pub const BUILD_DATETIME_UTC_RFC2822: &str = build_info::build_datetime_utc_rfc2822!();
pub const BUILD_DATETIME_UTC_RFC3339: &str = build_info::build_datetime_utc_rfc3339!();
pub const BUILD_DATETIME_LOCAL_FIXED: DateTime<FixedOffset> = build_info::build_datetime_local_fixed!();
pub const BUILD_DATETIME_LOCAL_FIXED_FORMAT: &str = build_info::build_datetime_local_fixed_format!("%d/%m/%Y %H:%M:%S %Z");
pub const BUILD_DATETIME_LOCAL_FIXED_RFC2822: &str = build_info::build_datetime_local_fixed_rfc2822!();
pub const BUILD_DATETIME_LOCAL_FIXED_RFC3339: &str = build_info::build_datetime_local_fixed_rfc3339!();
pub const BUILD_DATETIME_NAIVE: NaiveDateTime = build_info::build_datetime_naive!();
pub const BUILD_DATETIME_NAIVE_FORMAT: &str = build_info::build_datetime_naive_format!("%d/%m/%Y %H:%M:%S");
pub const BUILD_DATE_NAIVE: NaiveDate = build_info::build_date_naive!();
pub const BUILD_DATE_NAIVE_FORMAT: &str = build_info::build_date_naive_format!("%d/%m/%Y");
pub const BUILD_TIME_NAIVE: NaiveTime = build_info::build_time_naive!();
pub const BUILD_TIME_NAIVE_FORMAT: &str = build_info::build_time_naive_format!("%H:%M:%S");

fn main() {}
