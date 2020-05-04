use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::fmt::{Display, Formatter};
use url::Url;

const CALLBACK_HOST: &str = "x-callback-url";
pub const CALLBACK_PARAM_KEY_SOURCE: &str = "x-source";
pub const CALLBACK_PARAM_KEY_SUCCESS: &str = "x-success";
pub const CALLBACK_PARAM_KEY_ERROR: &str = "x-error";
pub const CALLBACK_PARAM_KEY_CANCEL: &str = "x-cancel";

pub type XCallbackParam = (String, String);

#[derive(Debug, Clone)]
pub struct XCallbackUrl {
    scheme: String,
    action: String,
    action_params: Vec<XCallbackParam>,
    callback_params: Vec<XCallbackParam>,
}

#[allow(dead_code)]
impl XCallbackUrl {
    pub fn parse(input: &str) -> Result<XCallbackUrl, Box<dyn Error>> {
        let url = Url::parse(input)?;

        if !url.host_str().eq(&Some(CALLBACK_HOST)) {
            return Err(Box::new(XCallbackError::InvalidHost(
                url.host_str().unwrap_or("").to_string(),
            )));
        }

        let scheme = url.scheme().to_string();
        let action = if !url.path().is_empty() {
            &url.path()[1..]
        } else {
            ""
        }
        .to_string();
        let (callback_params, action_params): (Vec<XCallbackParam>, Vec<XCallbackParam>) = url
            .query_pairs()
            .into_owned()
            .partition(XCallbackUrl::is_callback_param);

        Ok(XCallbackUrl {
            scheme,
            action,
            action_params,
            callback_params,
        })
    }

    pub fn new(scheme: &str) -> Self {
        XCallbackUrl {
            scheme: scheme.to_string(),
            action: "".to_string(),
            action_params: vec![],
            callback_params: vec![],
        }
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn set_scheme<T: ToString>(&mut self, scheme: T) {
        self.scheme = scheme.to_string();
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn set_action<T: ToString>(&mut self, action: T) {
        self.action = action.to_string();
    }

    pub fn action_params(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> {
        self.action_params
            .iter()
            .map(|(k, v)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(v.as_str())))
    }

    pub fn set_action_params<I, K, V>(&mut self, action_params: I)
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: ToString,
        V: ToString,
    {
        self.action_params = action_params
            .into_iter()
            .map(|i| {
                let (k, v) = i.borrow();
                (k.to_string(), v.to_string())
            })
            .collect();
    }

    pub fn append_action_param<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: ToString,
    {
        self.action_params
            .push((key.to_string(), value.to_string()));
    }

    pub fn callback_params(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> {
        self.callback_params
            .iter()
            .map(|(k, v)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(v.as_str())))
    }

    pub fn set_callback_params<I, K, V>(&mut self, callback_params: I)
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: ToString,
        V: ToString,
    {
        self.callback_params = callback_params
            .into_iter()
            .map(|i| {
                let (k, v) = i.borrow();
                (k.to_string(), v.to_string())
            })
            .collect();
    }

    pub fn source(&self) -> Option<String> {
        self.callback_params
            .iter()
            .find(|(k, _)| k == CALLBACK_PARAM_KEY_SOURCE)
            .map(|(_, v)| v.to_string())
    }

    pub fn to_url(&self) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(&format!(
            "{scheme}://{host}/{action}",
            host = CALLBACK_HOST,
            scheme = self.scheme,
            action = self.action,
        ))?;

        let parameters = self.action_params.iter().chain(self.callback_params.iter());
        url.query_pairs_mut().extend_pairs(parameters);
        Ok(url)
    }

    fn is_callback_param<T: AsRef<str>>((k, _): &(T, T)) -> bool {
        k.as_ref().starts_with("x-")
    }
}

impl ToString for XCallbackUrl {
    fn to_string(&self) -> String {
        self.to_url()
            .ok()
            .map(|u| u.to_string())
            .unwrap_or_else(|| "".to_string())
    }
}

// TODO - rethink this response if the params are the same
pub enum XCallbackResponse {
    Success {
        action_params: Vec<XCallbackParam>,
    },
    Error {
        action_params: Vec<XCallbackParam>,
    },
    Cancel {
        action_params: Vec<XCallbackParam>,
    },
}

pub trait XCallbackClient {
    fn execute(&self, url: &XCallbackUrl) -> Result<XCallbackResponse, Box<dyn Error>>;
}

#[derive(Debug)]
pub enum XCallbackError {
    InvalidHost(String),
    InvalidAction(String),
}

impl Display for XCallbackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            XCallbackError::InvalidHost(host) => {
                f.write_fmt(format_args!("Invalid host: {}", host))
            }
            XCallbackError::InvalidAction(action) => {
                f.write_fmt(format_args!("Invalid action: {}", action))
            }
        }
    }
}

impl Error for XCallbackError {}
