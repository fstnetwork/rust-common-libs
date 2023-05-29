use std::{
    borrow::Cow,
    ffi::{CStr, NulError},
    fmt,
    fmt::Write,
    num::TryFromIntError,
    path::Path,
    str::Utf8Error,
    time::Duration,
};

use snafu::Snafu;
use url::Url;

use pulsar_client_sys::ResultCode;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Ffi { source: Box<FfiError> },

    #[snafu(display("duration {duration:?} is not valid {unit}: {source}"))]
    InvalidDuration { duration: Box<Duration>, unit: Cow<'static, str>, source: FfiError },

    #[snafu(display("number is not valid: {source}"))]
    InvalidNumber { source: FfiError },

    #[snafu(display("path `{}` is not valid: {source}", path.display()))]
    InvalidPath { path: Cow<'static, Path>, source: FfiError },

    #[snafu(display("path `{c_str:?}` is not valid: {source}",))]
    InvalidPathCStr { c_str: Cow<'static, CStr>, source: FfiError },

    #[snafu(display("url `{url}` is not valid: {source}"))]
    InvalidUrl { url: Box<Url>, source: FfiError },

    #[snafu(display("consumer name `{consumer_name}` is not valid: {source}"))]
    InvalidConsumerName { consumer_name: Cow<'static, str>, source: FfiError },

    #[snafu(display("producer name `{producer_name}` is not valid: {source}"))]
    InvalidProducerName { producer_name: Cow<'static, str>, source: FfiError },

    #[snafu(display("subscription name `{subscription_name}` is not valid: {source}"))]
    InvalidSubscriptionName { subscription_name: Cow<'static, str>, source: FfiError },

    #[snafu(display("topics `{topic}` is not valid: {source}"))]
    InvalidTopic { topic: Cow<'static, str>, source: FfiError },

    #[snafu(display(
        "subscribe error for {} - `{subscription_name}`: {source}",
        ListDisplay(topics)
    ))]
    Subscribe { topics: Vec<String>, subscription_name: Cow<'static, str>, source: ResultCode },

    #[snafu(display("subscribe too many topics: {source}"))]
    SubscribeTooManyTopics { source: FfiError },

    #[snafu(display("create producer error for {topic}: {source}"))]
    CreateProducer { topic: String, source: ResultCode },

    #[snafu(display("lookup partitioned {topic}: {source}"))]
    LookupPartitionedTopic { topic: String, source: ResultCode },

    #[snafu(display("receive error: {source}"))]
    Receive { source: ResultCode },

    #[snafu(display("acknowledge error: {source}"))]
    Acknowledge { source: ResultCode },
}

impl From<FfiError> for Error {
    #[inline]
    fn from(source: FfiError) -> Self { Self::Ffi { source: Box::new(source) } }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FfiError {
    #[snafu(display("invalid C string: {source}",))]
    InvalidCString { source: NulError },

    #[snafu(display("invalid string: {source}"))]
    InvalidStringUtf8 { source: Utf8Error },

    #[snafu(display("invalid string: invalid utf-8 string"))]
    InvalidString,

    #[snafu(display("is not in {range}"))]
    NumberOutOfRange {
        number: Cow<'static, str>,
        range: Cow<'static, str>,
        source: TryFromIntError,
    },
}

struct ListDisplay<'a, I>(&'a Vec<I>)
where
    I: fmt::Display;

impl<I> fmt::Display for ListDisplay<'_, I>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();
        // handle empty
        let Some(first) = iter.next() else {
            return f.write_str("[]");
        };

        // handle single
        let Some(second) = iter.next() else {
            return first.fmt(f);
        };

        // handle multiple
        f.write_char('[')?;
        first.fmt(f)?;
        f.write_char(',')?;
        second.fmt(f)?;
        for item in iter {
            f.write_char(',')?;
            item.fmt(f)?;
        }
        f.write_char(']')
    }
}
