#[cfg(feature = "chrono")]
extern crate chrono;
extern crate proc_macro;
extern crate quote;
extern crate syn;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Datelike, Timelike, Utc, Local, TimeZone, FixedOffset};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::parse_macro_input;
use syn::parse::Parse;

use std::sync::{LazyLock, OnceLock};
#[cfg(feature = "git")]
use std::process::Command;

#[cfg(feature = "git")]
fn compile_error(message: &str) -> TokenStream {
  TokenStream::from(quote::quote!{ compile_error!(#message) })
}

#[cfg(feature = "git")]
fn git(args: &[&str]) -> Result<String, String> {
  match Command::new("git").args(args).output() {
    Ok(output) => match output.status.success() {
      true => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
      false => {
        let status = output.status.code().unwrap_or(1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git failed with status {status}: {stderr:?}"))
      }
    },
    Err(error) => Err(format!("git execution error: {error}"))
  }
}

#[cfg(feature = "git")]
static GIT_HASH: OnceLock<Result<String, String>> = OnceLock::new();

#[cfg(feature = "git")]
#[proc_macro]
pub fn git_hash(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_HASH.get_or_init(|| git(&["rev-parse", "HEAD"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "git")]
static GIT_HASH_SHORT: OnceLock<Result<String, String>> = OnceLock::new();

#[cfg(feature = "git")]
#[proc_macro]
pub fn git_hash_short(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_HASH_SHORT.get_or_init(|| git(&["rev-parse", "--short", "HEAD"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "git")]
static GIT_BRANCH: OnceLock<Result<String, String>> = OnceLock::new();

#[cfg(feature = "git")]
#[proc_macro]
pub fn git_branch(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_BRANCH.get_or_init(|| git(&["rev-parse", "--abbrev-ref", "HEAD"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "git")]
static GIT_ROOT: OnceLock<Result<String, String>> = OnceLock::new();

#[cfg(feature = "git")]
#[proc_macro]
pub fn git_root(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_ROOT.get_or_init(|| git(&["rev-parse", "--show-toplevel"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "chrono")]
static BUILD_DATETIME: LazyLock<DateTime<Utc>> = LazyLock::new(Utc::now);

#[cfg(feature = "chrono")]
fn get_build_datetime_utc() -> DateTime<Utc> {
  *BUILD_DATETIME
}

#[cfg(feature = "chrono")]
fn get_build_datetime_local_fixed() -> DateTime<FixedOffset> {
  BUILD_DATETIME.with_timezone(&Local).fixed_offset()
}

#[cfg(feature = "chrono")]
fn build_chrono<F, Tz, I, O>(input: TokenStream, get: impl FnOnce() -> DateTime<Tz>, f: F) -> TokenStream
where F: FnOnce(I, syn::Ident, DateTime<Tz>) -> O, Tz: TimeZone, I: Parse, O: ToTokens {
  let input = parse_macro_input!(input as I);
  let build_info = syn::Ident::new("_build_info", Span::call_site());
  let token_stream = f(input, build_info.clone(), get());

  TokenStream::from(quote::quote!{
    const {
      #[allow(unused_extern_crates, clippy::useless_attribute)]
      extern crate build_info as #build_info;
      #token_stream
    }
  })
}

#[cfg(feature = "chrono")]
fn build_chrono_bare<F, Tz, I, O>(input: TokenStream, get: impl FnOnce() -> DateTime<Tz>, f: F) -> TokenStream
where F: FnOnce(I, DateTime<Tz>) -> O, Tz: TimeZone, I: Parse, O: ToTokens {
  let input = parse_macro_input!(input as I);
  let token_stream = f(input, get());
  TokenStream::from(token_stream.into_token_stream())
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_utc(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_datetime_utc, |syn::parse::Nothing, build_info, datetime| {
    let naive_datetime = datetime.naive_utc();
    let naive_date_days = naive_datetime.date().num_days_from_ce();
    let naive_time_secs = naive_datetime.time().num_seconds_from_midnight();
    let naive_time_frac = naive_datetime.time().nanosecond();

    quote::quote! {
      #build_info::chrono::DateTime::<#build_info::chrono::Utc>::from_naive_utc_and_offset(
        #build_info::chrono::NaiveDateTime::new(
          #build_info::chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap(),
          #build_info::chrono::NaiveTime::from_num_seconds_from_midnight_opt(#naive_time_secs, #naive_time_frac).unwrap()
        ),
        #build_info::chrono::Utc
      )
    }
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_utc_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_utc, |input: syn::LitStr, datetime| {
    datetime.format(&input.value()).to_string()
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_utc_rfc2822(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_utc, |syn::parse::Nothing, datetime| datetime.to_rfc2822())
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_utc_rfc3339(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_utc, |syn::parse::Nothing, datetime| datetime.to_rfc3339())
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_local_fixed(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_datetime_local_fixed, |syn::parse::Nothing, build_info, datetime| {
    let local_minus_utc = datetime.offset().local_minus_utc();
    let naive_datetime = datetime.naive_utc();
    let naive_date_days = naive_datetime.date().num_days_from_ce();
    let naive_time_secs = naive_datetime.time().num_seconds_from_midnight();
    let naive_time_frac = naive_datetime.time().nanosecond();

    quote::quote! {
      #build_info::chrono::DateTime::<#build_info::chrono::FixedOffset>::from_naive_utc_and_offset(
        #build_info::chrono::NaiveDateTime::new(
          #build_info::chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap(),
          #build_info::chrono::NaiveTime::from_num_seconds_from_midnight_opt(#naive_time_secs, #naive_time_frac).unwrap()
        ),
        #build_info::chrono::FixedOffset::west_opt(#local_minus_utc).unwrap()
      )
    }
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_local_fixed_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_local_fixed, |input: syn::LitStr, datetime| {
    datetime.format(&input.value()).to_string()
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_local_fixed_rfc2822(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_local_fixed, |syn::parse::Nothing, datetime| datetime.to_rfc2822())
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_local_fixed_rfc3339(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_local_fixed, |syn::parse::Nothing, datetime| datetime.to_rfc3339())
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_naive(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_datetime_utc, |syn::parse::Nothing, build_info, datetime| {
    let naive_datetime = datetime.naive_utc();
    let naive_date_days = naive_datetime.date().num_days_from_ce();
    let naive_time_secs = naive_datetime.time().num_seconds_from_midnight();
    let naive_time_frac = naive_datetime.time().nanosecond();

    quote::quote! {
      #build_info::chrono::NaiveDateTime::new(
        #build_info::chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap(),
        #build_info::chrono::NaiveTime::from_num_seconds_from_midnight_opt(#naive_time_secs, #naive_time_frac).unwrap()
      )
    }
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_datetime_naive_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_utc, |input: syn::LitStr, datetime| {
    let naive_datetime = datetime.naive_utc();
    naive_datetime.format(&input.value()).to_string()
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_date_naive(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_datetime_utc, |syn::parse::Nothing, build_info, datetime| {
    let naive_datetime = datetime.naive_utc();
    let naive_date_days = naive_datetime.date().num_days_from_ce();

    quote::quote! {
      #build_info::chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap()
    }
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_date_naive_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_utc, |input: syn::LitStr, datetime| {
    let naive_datetime = datetime.naive_utc();
    naive_datetime.date().format(&input.value()).to_string()
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_time_naive(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_datetime_utc, |syn::parse::Nothing, build_info, datetime| {
    let naive_datetime = datetime.naive_utc();
    let naive_time_secs = naive_datetime.time().num_seconds_from_midnight();
    let naive_time_frac = naive_datetime.time().nanosecond();

    quote::quote! {
      #build_info::chrono::NaiveTime::from_num_seconds_from_midnight_opt(#naive_time_secs, #naive_time_frac).unwrap()
    }
  })
}

#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_time_naive_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_datetime_utc, |input: syn::LitStr, datetime| {
    let naive_datetime = datetime.naive_utc();
    naive_datetime.time().format(&input.value()).to_string()
  })
}

#[cfg(test)]
mod tests {
  #[cfg(feature = "chrono")]
  use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate, NaiveTime, Datelike, Timelike};

  #[cfg(feature = "chrono")]
  #[test]
  fn test_roundtrips() {
    let now = Utc::now();
    test_roundtrip(now);
  }

  #[cfg(feature = "chrono")]
  fn test_roundtrip(datetime: DateTime<Utc>) {
    let naive_datetime = datetime.naive_utc();
    let naive_date_days = naive_datetime.date().num_days_from_ce();
    let naive_time_secs = naive_datetime.time().num_seconds_from_midnight();
    let naive_time_frac = naive_datetime.time().nanosecond();

    let datetime2 = DateTime::<Utc>::from_naive_utc_and_offset(
      NaiveDateTime::new(
        NaiveDate::from_num_days_from_ce_opt(naive_date_days).unwrap(),
        NaiveTime::from_num_seconds_from_midnight_opt(naive_time_secs, naive_time_frac).unwrap()
      ),
      Utc
    );

    assert_eq!(datetime, datetime2);
  }
}
