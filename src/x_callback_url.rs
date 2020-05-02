use std::borrow::{Borrow, Cow};
use url::Url;

const CALLBACK_HOST: &str = "x-callback-url";
pub const CALLBACK_PARAM_KEY_SOURCE: &str = "x-source";
pub const CALLBACK_PARAM_KEY_SUCCESS: &str = "x-success";
pub const CALLBACK_PARAM_KEY_ERROR: &str = "x-error";
pub const CALLBACK_PARAM_KEY_CANCEL: &str = "x-cancel";

#[derive(Debug, Clone)]
pub struct XCallbackUrl {
    url: Url,
}

#[allow(dead_code)]
impl XCallbackUrl {
    pub fn new(scheme: &str) -> Self {
        // The stand-in `action` in the path serves to avoid a problem where Url
        // parses the Url successfully, but when a path is set later, the Url
        // serialization is missing the '/` between the host and path.
        let mut url = Url::parse(&format!(
            "{scheme}://{host}/action",
            scheme = scheme,
            host = CALLBACK_HOST,
        ))
        .unwrap();
        url.set_path("");

        XCallbackUrl { url }
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub fn set_scheme(&mut self, scheme: &str) -> Result<(), ()> {
        self.url.set_scheme(scheme)
    }

    pub fn action(&self) -> &str {
        &self.url.path()[1..]
    }

    pub fn set_action(&mut self, action: &str) {
        self.url.set_path(action);
    }

    pub fn params(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> {
        self.url.query_pairs()
    }

    pub fn set_params<I, K, V>(&mut self, params: I)
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.url.query_pairs_mut().clear().extend_pairs(params);
    }

    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    pub fn to_url(&self) -> Url {
        self.url.clone()
    }
}
