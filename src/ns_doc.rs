//! Nightscout documents with fields that nightscout-rs leaves out.

use chrono::{DateTime, SecondsFormat, Utc};
use nightscout::Entry;
use serde::Serialize;
use serde::ser::{Error as _, Serializer};
use serde_json::Value;

/// An `Entry` serialized with `dateString` and `sysTime`.
///
/// Nightscout's v3 API stores an entry as posted and never derives these two
/// fields from `date`, but v1 API clients and `find[dateString]` queries
/// expect them on every entry.
pub struct EntryDocument<'a>(pub &'a Entry);

impl Serialize for EntryDocument<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serde_json::to_value(self.0).map_err(S::Error::custom)?;
        if let (Some(fields), Some(at)) = (
            value.as_object_mut(),
            DateTime::<Utc>::from_timestamp_millis(self.0.base.date),
        ) {
            let iso = at.to_rfc3339_opts(SecondsFormat::Millis, true);
            fields.insert("dateString".to_string(), Value::String(iso.clone()));
            fields.insert("sysTime".to_string(), Value::String(iso));
        }
        value.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nightscout::DocumentBase;
    use rust_decimal::Decimal;

    fn entry(date: i64) -> Entry {
        Entry {
            base: DocumentBase {
                identifier: Some(format!("cgm-{}", date / 1000)),
                date,
                utc_offset: None,
                app: "test".to_string(),
                device: None,
                id_internal: None,
                srv_created: None,
                subject: None,
                srv_modified: None,
                modified_by: None,
                is_valid: None,
                is_read_only: None,
            },
            type_: Some("sgv".to_string()),
            sgv: Some(Decimal::from(121)),
            direction: None,
            noise: None,
            filtered: None,
            unfiltered: None,
            rssi: None,
            units: Some("mg/dl".to_string()),
        }
    }

    #[test]
    fn adds_date_string_and_sys_time_in_utc() {
        let doc = serde_json::to_value(EntryDocument(&entry(1_790_811_617_000))).unwrap();
        assert_eq!(doc["dateString"], "2026-09-30T23:40:17.000Z");
        assert_eq!(doc["sysTime"], "2026-09-30T23:40:17.000Z");
    }

    #[test]
    fn keeps_the_entry_fields() {
        let doc = serde_json::to_value(EntryDocument(&entry(1_790_811_617_000))).unwrap();
        assert_eq!(doc["identifier"], "cgm-1790811617");
        assert_eq!(doc["date"], 1_790_811_617_000_i64);
        assert_eq!(doc["sgv"], 121);
        assert_eq!(doc["type"], "sgv");
        assert!(doc.get("direction").is_none(), "None fields stay omitted");
    }
}
