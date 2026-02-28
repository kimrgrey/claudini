use serde_json::Value;

/// Fields that are account-specific and should NOT be synced between profiles.
const ACCOUNT_SPECIFIC_FIELDS: &[&str] = &[
    "oauthAccount",
    "userID",
    "s1mAccessCache",
    "groveConfigCache",
    "passesEligibilityCache",
    "hasShownOpus46Notice",
    "cachedGrowthBookFeatures",
    "cachedExtraUsageDisabledReason",
    "penguinModeOrgEnabled",
    "clientDataCache",
    "claudeCodeFirstTokenDate",
    "hasVisitedExtraUsage",
    "hasVisitedPasses",
    "passesLastSeenRemaining",
];

/// Copies all shared (non-account-specific) fields from `from` into `to`.
///
/// Account-specific fields in `to` are preserved. All other fields from `from`
/// overwrite whatever is in `to`.
pub fn sync_shared_fields(from: &Value, to: &mut Value) {
    let (Some(from_obj), Some(to_obj)) = (from.as_object(), to.as_object_mut()) else {
        return;
    };

    for (key, value) in from_obj {
        if !ACCOUNT_SPECIFIC_FIELDS.contains(&key.as_str()) {
            to_obj.insert(key.clone(), value.clone());
        }
    }
}
