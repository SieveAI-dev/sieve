//! Timestamp 序列化辅助：UTC + 毫秒精度 + `Z` 后缀（SPEC-005 §4A）。
//!
//! 用法：给 `DateTime<Utc>` 字段加 `#[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]`。
//!
//! 输出示例：`"2026-05-03T12:34:56.789Z"`（ISO 8601 + 毫秒 + Z 后缀）。
//!
//! 反序列化不变（Chrono 默认支持多种格式）。

use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serializer;

/// 将 `DateTime<Utc>` 序列化为 RFC 3339 格式，毫秒精度，Z 后缀。
///
/// 关联：SPEC-005 §4A（Timestamp 序列化约束）。
pub fn serialize_utc_millis<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = dt.to_rfc3339_opts(SecondsFormat::Millis, true);
    serializer.serialize_str(&s)
}

/// 将 `Option<DateTime<Utc>>` 序列化为 RFC 3339 格式（毫秒 + Z）或 null。
pub fn serialize_opt_utc_millis<S>(
    dt: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(t) => serialize_utc_millis(t, serializer),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Wrapper {
        #[serde(serialize_with = "serialize_utc_millis")]
        ts: DateTime<Utc>,
    }

    /// 序列化结果含 Z 后缀和毫秒精度。
    #[test]
    fn serialize_has_z_suffix_and_millis() {
        let dt = Utc.with_ymd_and_hms(2026, 5, 3, 12, 34, 56).unwrap()
            + chrono::Duration::milliseconds(789);
        let w = Wrapper { ts: dt };
        let json = serde_json::to_string(&w).unwrap();
        // 期望含 "Z" 后缀和 ".789"
        assert!(
            json.contains("\"2026-05-03T12:34:56.789Z\""),
            "expected millis+Z in json, got: {json}"
        );
    }

    /// 整秒时仍补全 .000。
    #[test]
    fn serialize_whole_second_has_000_millis() {
        let dt = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let w = Wrapper { ts: dt };
        let json = serde_json::to_string(&w).unwrap();
        assert!(
            json.contains(".000Z"),
            "whole-second timestamp should serialize as .000Z, got: {json}"
        );
    }
}
