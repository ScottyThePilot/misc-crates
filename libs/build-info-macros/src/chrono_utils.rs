#[cfg(feature = "chrono")]
use chrono::{DateTime, NaiveDateTime, NaiveDate, Datelike, NaiveTime, Timelike, Utc, TimeZone, FixedOffset};
use quote::ToTokens;

#[derive(Debug, Clone)]
pub struct ToTokensDateTime<Tz: TimeZone> {
  build_info: syn::Ident,
  datetime: DateTime<Tz>
}

impl<Tz: TimeZone> ToTokensDateTime<Tz> {
  pub fn new(build_info: syn::Ident, datetime: DateTime<Tz>) -> Self {
    ToTokensDateTime { build_info, datetime }
  }
}

impl ToTokens for ToTokensDateTime<Utc> {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> proc_macro2::TokenStream {
    let build_info = &self.build_info;
    let tokens_naive_datetime = ToTokensNaiveDateTime::new(build_info.clone(), self.datetime.naive_utc());

    quote::quote! {
      #build_info::chrono::DateTime::<#build_info::chrono::Utc>::from_naive_utc_and_offset(
        #tokens_naive_datetime,
        #build_info::chrono::Utc
      )
    }
  }
}

impl ToTokens for ToTokensDateTime<FixedOffset> {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> proc_macro2::TokenStream {
    let build_info = &self.build_info;
    let local_minus_utc = self.datetime.offset().local_minus_utc();
    let tokens_naive_datetime = ToTokensNaiveDateTime::new(build_info.clone(), self.datetime.naive_utc());

    quote::quote! {
      #build_info::chrono::DateTime::<#build_info::chrono::FixedOffset>::from_naive_utc_and_offset(
        #tokens_naive_datetime,
        #build_info::chrono::FixedOffset::west_opt(#local_minus_utc).unwrap()
      )
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToTokensNaiveDateTime {
  build_info: syn::Ident,
  naive_datetime: NaiveDateTime
}

impl ToTokensNaiveDateTime {
  pub fn new(build_info: syn::Ident, naive_datetime: NaiveDateTime) -> Self {
    ToTokensNaiveDateTime { build_info, naive_datetime }
  }
}

impl ToTokens for ToTokensNaiveDateTime {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> proc_macro2::TokenStream {
    let build_info = &self.build_info;
    let tokens_naive_date = ToTokensNaiveDate::new(build_info.clone(), self.naive_datetime.date());
    let tokens_naive_time = ToTokensNaiveTime::new(build_info.clone(), self.naive_datetime.time());

    quote::quote! {
      #build_info::chrono::NaiveDateTime::new(#tokens_naive_date, #tokens_naive_time)
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToTokensNaiveDate {
  build_info: syn::Ident,
  naive_nate: NaiveDate
}

impl ToTokensNaiveDate {
  pub fn new(build_info: syn::Ident, naive_nate: NaiveDate) -> Self {
    ToTokensNaiveDate { build_info, naive_nate }
  }
}

impl ToTokens for ToTokensNaiveDate {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> proc_macro2::TokenStream {
    let build_info = &self.build_info;
    let naive_date_days = self.naive_nate.num_days_from_ce();

    quote::quote! {
      #build_info::chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap()
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToTokensNaiveTime {
  build_info: syn::Ident,
  naive_time: NaiveTime
}

impl ToTokensNaiveTime {
  pub fn new(build_info: syn::Ident, naive_time: NaiveTime) -> Self {
    ToTokensNaiveTime { build_info, naive_time }
  }
}

impl ToTokens for ToTokensNaiveTime {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> proc_macro2::TokenStream {
    let build_info = &self.build_info;
    let naive_time_secs = self.naive_time.num_seconds_from_midnight();
    let naive_time_frac = self.naive_time.nanosecond();

    quote::quote! {
      #build_info::chrono::NaiveTime::from_num_seconds_from_midnight_opt(#naive_time_secs, #naive_time_frac).unwrap()
    }
  }
}
