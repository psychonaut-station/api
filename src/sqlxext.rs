//! SQLx extensions and custom type mappings.
//!
//! Provides custom wrapper types for MySQL DATE and DATETIME columns that
//! automatically convert to formatted strings when returned from queries.

use std::fmt::{self, Display, Formatter};

use sqlx::{
    Decode, MySql, Type,
    error::BoxDynError,
    mysql::{MySqlTypeInfo, MySqlValueRef},
    types::chrono::{NaiveDate, NaiveDateTime},
};

/// Wrapper for MySQL DATETIME type that automatically formats to string.
///
/// This type implements SQLx's `Type` and `Decode` traits to handle MySQL DATETIME columns,
/// and provides automatic conversion to a formatted string in the format "YYYY-MM-DD HH:MM:SS".
pub struct DateTime(NaiveDateTime);

impl Type<MySql> for DateTime {
    fn type_info() -> MySqlTypeInfo {
        NaiveDateTime::type_info()
    }
}

impl<'r> Decode<'r, MySql> for DateTime {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        NaiveDateTime::decode(value).map(DateTime)
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

        write!(f, "{}", self.0.format(DATETIME_FORMAT))
    }
}

/// Wrapper for MySQL DATE type that automatically formats to string.
///
/// This type implements SQLx's `Type` and `Decode` traits to handle MySQL DATE columns,
/// and provides automatic conversion to a formatted string in the format "YYYY-MM-DD".
pub struct Date(NaiveDate);

impl Type<MySql> for Date {
    fn type_info() -> MySqlTypeInfo {
        NaiveDate::type_info()
    }
}

impl<'r> Decode<'r, MySql> for Date {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        NaiveDate::decode(value).map(Date)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        const DATE_FORMAT: &str = "%Y-%m-%d";

        write!(f, "{}", self.0.format(DATE_FORMAT))
    }
}
