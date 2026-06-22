use validator::ValidationError;
use chrono::{DateTime, Utc};


pub fn ensure_future_date(date: &DateTime<Utc>) -> Result<(), ValidationError> {
    if *date <= Utc::now() {
        return Err(ValidationError::new("date_in_past"))
    }

    Ok(())
}
