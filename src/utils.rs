use scraper::Selector;

/// Returns a `Selector` from a given `&str`
pub fn selector(selector: &str) -> Selector {
    Selector::parse(selector).unwrap_or_else(|err| panic!("Error parsing selector: {}", err))
}
