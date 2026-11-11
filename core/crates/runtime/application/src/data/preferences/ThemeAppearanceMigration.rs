//! Versioned startup migration and repair from loose appearance preferences to ordinary saved themes.
//! These are business transactions so the catalog, selection, migration version and repaired
//! ordinary appearance sync atomically.
use std::sync::Arc;

use operit_host_api::RuntimeStorageHost;
use operit_store::PreferencesDataStore::{stringPreferencesKey, PreferencesDataStoreError};

const VERSION_KEY: &str = "theme_appearance_migration_version";
const VERSION: u32 = 1;

/// Runs after legacy character-theme import and before the runtime becomes ready.
/// The one-time migration adopts released appearance preferences; every later startup reconciles
/// the persisted active selection with the ordinary appearance so a single-sided write from
/// another owner cannot keep the theme library unreadable. Ordinary preference readers and the
/// theme library never invoke this migration.
pub(crate) fn migrate(
    storage: Arc<dyn RuntimeStorageHost>,
) -> Result<(), PreferencesDataStoreError> {
    let store = super::UserPreferencesManager::UserPreferencesManager::store(storage);
    store.try_edit_result(|preferences| {
        let key = stringPreferencesKey(VERSION_KEY);
        let version = preferences.getTyped::<u32>(&key)?.unwrap_or(0);
        match version {
            VERSION => {}
            0 => {
                super::ThemeConfigManager::migrateUnboundAppearance(preferences)?;
                preferences.set(&key, VERSION);
            }
            from => {
                return Err(PreferencesDataStoreError::Message(format!(
                    "Unsupported theme appearance migration version: {from}"
                )))
            }
        }
        super::ThemeConfigManager::reconcileActiveAppearance(preferences)
    })
}
