use std::borrow::{Borrow, Cow};
use url::Url;
use std::error::Error;

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
    pub fn parse(input: &str) -> Result<XCallbackUrl, Box<dyn Error>> {
        // FIXME - return errors
        let url = Url::parse(input).unwrap();
        assert_eq!(url.host_str().unwrap(), CALLBACK_HOST);
        assert!(!url.cannot_be_a_base());
        Ok(XCallbackUrl { url })
    }

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
        &self.url.path()[1..] // FIXME - what if there is no action?
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

    pub fn action_params(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> {
        self.url
            .query_pairs()
            .filter(|p| !XCallbackUrl::is_callback_param(p))
    }

    pub fn source(&self) -> Option<String> {
        self.url
            .query_pairs()
            .find(|(k, _)| k == CALLBACK_PARAM_KEY_SOURCE)
            .map(|(_, v)| v.to_string())
    }

    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    pub fn to_url(&self) -> Url {
        self.url.clone()
    }

    fn is_callback_param<T: AsRef<str>>((k, _): &(T, T)) -> bool {
        k.as_ref().starts_with("x-")
    }
}

pub enum XCallbackResponse {
    Success { params: Vec<(String, String)> },
    Error { params: Vec<(String, String)> },
    Cancel { params: Vec<(String, String)> },
}

pub trait XCallbackClient {
    fn execute(&self, url: &XCallbackUrl) -> Result<XCallbackResponse, Box<dyn Error>>;
}
