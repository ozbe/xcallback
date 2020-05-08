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
    source: Option<String>,
    success: Option<String>,
    error: Option<String>,
    cancel: Option<String>,
}

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

        let mut callback_url = XCallbackUrl {
            scheme,
            action,
            action_params: vec![],
            source: None,
            success: None,
            error: None,
            cancel: None,
        };

        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                CALLBACK_PARAM_KEY_SOURCE => callback_url.source = Some(v.to_string()),
                CALLBACK_PARAM_KEY_SUCCESS => callback_url.success = Some(v.to_string()),
                CALLBACK_PARAM_KEY_ERROR => callback_url.error = Some(v.to_string()),
                CALLBACK_PARAM_KEY_CANCEL => callback_url.cancel = Some(v.to_string()),
                _ => callback_url
                    .action_params
                    .push((k.to_string(), v.to_string())),
            }
        }

        Ok(callback_url)
    }

    pub fn new(scheme: &str) -> Self {
        XCallbackUrl {
            scheme: scheme.to_string(),
            action: "".to_string(),
            action_params: vec![],
            source: None,
            success: None,
            error: None,
            cancel: None,
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

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn set_source<T: ToString>(&mut self, source: Option<T>) {
        self.source = source.map(|s| s.to_string());
    }

    pub fn success(&self) -> Option<&str> {
        self.success.as_deref()
    }

    pub fn set_success<T: ToString>(&mut self, success: Option<T>) {
        self.success = success.map(|s| s.to_string());
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error<T: ToString>(&mut self, error: Option<T>) {
        self.error = error.map(|s| s.to_string());
    }

    pub fn cancel(&self) -> Option<&str> {
        self.cancel.as_deref()
    }

    pub fn set_cancel<T: ToString>(&mut self, cancel: Option<T>) {
        self.cancel = cancel.map(|s| s.to_string());
    }

    pub fn to_url(&self) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(&format!(
            "{scheme}://{host}/{action}",
            host = CALLBACK_HOST,
            scheme = self.scheme,
            action = self.action,
        ))?;

        if !self.action_params.is_empty() {
            url.query_pairs_mut().extend_pairs(&self.action_params);
        }

        let callback_params: Vec<_> = vec![
            (CALLBACK_PARAM_KEY_SOURCE, &self.source),
            (CALLBACK_PARAM_KEY_SUCCESS, &self.success),
            (CALLBACK_PARAM_KEY_ERROR, &self.error),
            (CALLBACK_PARAM_KEY_CANCEL, &self.cancel),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
        .collect();

        if !callback_params.is_empty() {
            url.query_pairs_mut().extend_pairs(callback_params);
        }

        Ok(url)
    }
}

#[cfg(test)]
mod test {
    mod x_callback_url {
        use crate::x_callback_url::XCallbackUrl;

        #[test]
        fn test() {
            let input = "callback://x-callback-url/action\
                    ?key=value\
                    &x-success=callback%3A%2F%2Fx-callback-success";

            let url = XCallbackUrl::parse(input).unwrap();

            assert_eq!("callback", url.scheme());
            assert_eq!("action", url.action());
            assert_eq!(
                vec![("key".to_string(), "value".to_string())],
                url.action_params()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                Some("callback://x-callback-success".to_string()),
                url.success
            );
            assert_eq!(url.to_string(), input);
        }

        #[test]
        fn test_no_params() {
            let input = "callback://x-callback-url/action";

            let url = XCallbackUrl::parse(input).unwrap();

            assert_eq!("callback", url.scheme());
            assert_eq!("action", url.action());
            assert_eq!(url.action_params().count(), 0);
            assert_eq!(url.source, None);
            assert_eq!(url.success, None);
            assert_eq!(url.error, None);
            assert_eq!(url.cancel, None);
            assert_eq!(url.to_string(), input);
        }

        #[test]
        fn test_no_action_params() {
            let input = "callback://x-callback-url/action\
                ?x-success=callback%3A%2F%2Fx-callback-success";

            let url = XCallbackUrl::parse(input).unwrap();

            assert_eq!("callback", url.scheme());
            assert_eq!("action", url.action());
            assert_eq!(url.action_params().count(), 0);
            assert_eq!(
                Some("callback://x-callback-success".to_string()),
                url.success
            );
            assert_eq!(url.to_string(), input);
        }

        #[test]
        fn test_no_callback_params() {
            let input = "callback://x-callback-url/action\
                ?key=value";

            let url = XCallbackUrl::parse(input).unwrap();

            assert_eq!("callback", url.scheme());
            assert_eq!("action", url.action());
            assert_eq!(
                vec![("key".to_string(), "value".to_string())],
                url.action_params()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<_>>()
            );
            assert_eq!(url.source, None);
            assert_eq!(url.success, None);
            assert_eq!(url.error, None);
            assert_eq!(url.cancel, None);
            assert_eq!(url.to_string(), input);
        }

        // test action, scheme, and params
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

pub struct XCallbackResponse {
    pub status: XCallbackStatus,
    pub action_params: Vec<XCallbackParam>,
}

pub enum XCallbackStatus {
    Success,
    Error,
    Cancel,
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
