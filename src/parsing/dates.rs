use chrono::prelude::*;
use chrono::ParseResult;
use log::*;

/// The date format used in the GoodReads data.
///
/// This almost RFC2822, but hte year is in the wrong place.
const GR_DATE_FMT: &'static str = "%A %B %d %H:%M:%S %z %Y";

/// Parse a GoodReads date.
pub fn parse_gr_date(s: &str) -> ParseResult<NaiveDateTime> {
    let date = DateTime::parse_from_str(s, GR_DATE_FMT)?.naive_utc();
    Ok(date)
}

/// Check a date and convert to a timestamp.
pub fn check_ts(
    context: &'static str,
    cutoff: i32,
) -> impl Fn(NaiveDateTime) -> Option<NaiveDateTime> {
    move |date| {
        if date.year() < cutoff {
            warn!("{} is ancient: {}", context, date);
            None
        } else {
            Some(date)
        }
    }
}
