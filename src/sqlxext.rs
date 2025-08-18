use sqlx::{
    Decode, MySql, Type,
    error::BoxDynError,
    mysql::{MySqlTypeInfo, MySqlValueRef},
    types::chrono::{NaiveDate, NaiveDateTime},
};

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

impl From<DateTime> for String {
    fn from(val: DateTime) -> Self {
        const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

        val.0.format(DATETIME_FORMAT).to_string()
    }
}

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

impl From<Date> for String {
    fn from(val: Date) -> Self {
        const DATE_FORMAT: &str = "%Y-%m-%d";

        val.0.format(DATE_FORMAT).to_string()
    }
}
