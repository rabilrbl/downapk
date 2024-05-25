use core::fmt;
use indicatif::style::TemplateError;
use reqwest::Error as ReqwestError;
use scraper::error::SelectorErrorKind as ScraperSelectorErrorKind;

#[derive(Debug)]
pub enum DownApkError<'a> {
    Reqwest(ReqwestError),
    Selector(ScraperSelectorErrorKind<'a>),
    Template(TemplateError),
    IoError(std::io::Error),
    Other(String),
}

impl fmt::Display for DownApkError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownApkError::Reqwest(e) => write!(f, "Reqwest error: {}", e),
            DownApkError::Selector(e) => write!(f, "Selector error: {}", e),
            DownApkError::Template(e) => write!(f, "Template error: {}", e),
            DownApkError::IoError(e) => write!(f, "IO error: {}", e),
            DownApkError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<ReqwestError> for DownApkError<'_> {
    fn from(e: ReqwestError) -> Self {
        DownApkError::Reqwest(e)
    }
}

impl<'a> From<ScraperSelectorErrorKind<'a>> for DownApkError<'a> {
    fn from(e: ScraperSelectorErrorKind<'a>) -> Self {
        DownApkError::Selector(e)
    }
}

impl From<TemplateError> for DownApkError<'_> {
    fn from(e: TemplateError) -> Self {
        DownApkError::Template(e)
    }
}

impl From<String> for DownApkError<'_> {
    fn from(e: String) -> Self {
        DownApkError::Other(e)
    }
}

impl From<&str> for DownApkError<'_> {
    fn from(e: &str) -> Self {
        DownApkError::Other(e.to_string())
    }
}

impl From<std::io::Error> for DownApkError<'_> {
    fn from(e: std::io::Error) -> Self {
        DownApkError::IoError(e)
    }
}

impl std::error::Error for DownApkError<'static> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownApkError::Reqwest(e) => Some(e),
            DownApkError::Selector(e) => Some(e),
            DownApkError::Template(e) => Some(e),
            DownApkError::IoError(e) => Some(e),
            DownApkError::Other(_) => None,
        }
    }
}
