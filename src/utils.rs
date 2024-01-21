use crate::errors::DownApkError;
use scraper::Selector;

/// Returns a `Selector` from a given `&str`
pub fn selector(selector: &str) -> Result<Selector, DownApkError> {
    Selector::parse(selector).map_err(|e| e.into())
}
