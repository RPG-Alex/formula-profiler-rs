use mascot_rs::prelude::MascotGenericFormatMetadata;

pub(crate) fn metadata_value(metadata: &MascotGenericFormatMetadata, key: &str) -> String {
    let value = metadata.arbitrary_metadata_value(key).unwrap_or_default();

    clean_metadata_value(value)
}

pub(crate) fn clean_metadata_value(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() { "UNKNOWN".to_string() } else { value.to_string() }
}

pub(crate) fn optional_debug_label<T: std::fmt::Debug>(value: Option<T>) -> String {
    value.map(|inner| format!("{inner:?}")).unwrap_or_else(|| "UNKNOWN".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_metadata_value_preserves_real_values() {
        assert_eq!(clean_metadata_value("GNPS"), "GNPS");
        assert_eq!(clean_metadata_value("  GNPS  "), "GNPS");
    }

    #[test]
    fn clean_metadata_value_maps_empty_values_to_unknown() {
        assert_eq!(clean_metadata_value(""), "UNKNOWN");
        assert_eq!(clean_metadata_value("   "), "UNKNOWN");
    }

    #[test]
    fn optional_debug_label_formats_some_values_without_option_wrapper() {
        assert_eq!(optional_debug_label(Some(42)), "42");
        assert_eq!(optional_debug_label(Some("Positive")), "\"Positive\"");
    }

    #[test]
    fn optional_debug_label_maps_none_to_unknown() {
        let value: Option<u8> = None;

        assert_eq!(optional_debug_label(value), "UNKNOWN");
    }
}
