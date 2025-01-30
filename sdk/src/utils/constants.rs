use std::time::Duration;

// Stronghold keys
pub const STRONGHOLD_KEY_HEDERA_PASSWORD: &str = "HEDERA_PASSWORD";
pub const STRONGHOLD_KEY_HEDERA_USERNAME: &str = "HEDERA_USERNAME";
pub const STRONGHOLD_DOC_KEYS: &str = "streams_doc_keys";
pub const STRONGHOLD_SIG_KEYS: &str = "streams_sig_keys";
pub const STRONGHOLD_KE_KEYS: &str = "streams_ke_keys";

// Identity fragments
pub const DID_FRAGMENT_HEDERA_DID: &str = "hedera-did";

// Urls
pub const LOCAL_API: &str = "http://localhost:1111";
pub const RETRIEVER_API: &str = "http://localhost:9000";
pub const GUARDIAN_API: &str = "http://guardian.demia-nodes.net/api/v1";
pub const SECRETS_API: &str = "https://auth.demia-testing-domain.com/realms/DemiaTest";

// Timeouts
pub const API_TIMEOUT: Duration = Duration::from_secs(10);

// Buckets
pub const PROTECTED_BUCKET_PATH: &str = "stronghold-snapshots";
pub const PROTECTED_BUCKET_PATH_TEST: &str = "staging-user-site-storage";
pub const PUBLIC_BUCKET_PATH: &str = "demia-public";

pub const fn bucket_path(production: bool) -> &'static str {
    match production {
        true => PROTECTED_BUCKET_PATH,
        false => PROTECTED_BUCKET_PATH_TEST,
    }
}
