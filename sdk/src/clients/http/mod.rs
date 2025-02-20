mod api;
mod guardian;
mod http_client;
mod retriever;

pub use api::ApiClient;
pub use guardian::{GuardianApiClient, GuardianClient};
pub(crate) use http_client::*;
pub use retriever::RetrieverApi;

pub(crate) fn query_tuples_to_query_string(
    tuples: impl IntoIterator<Item = Option<(&'static str, String)>>,
) -> Option<String> {
    let query = tuples
        .into_iter()
        .filter_map(|tuple| tuple.map(|(key, value)| format!("{}={}", key, value)))
        .collect::<Vec<_>>();

    if query.is_empty() { None } else { Some(query.join("&")) }
}
