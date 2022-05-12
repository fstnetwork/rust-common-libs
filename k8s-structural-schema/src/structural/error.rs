use std::borrow::Cow;

use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("CustomResourceDefinition is invalid: {reason}"))]
    InvalidCustomResourceDefinition { reason: Cow<'static, str>, backtrace: Backtrace },
}
