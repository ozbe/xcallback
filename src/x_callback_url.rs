use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::fmt::{Display, Formatter};
use url::Url;
use std::iter::FromIterator;

const CALLBACK_HOST: &str = "x-callback-url";
pub const CALLBACK_PARAM_KEY_SOURCE: &str = "x-source";
pub const CALLBACK_PARAM_KEY_SUCCESS: &str = "x-success";
pub const CALLBACK_PARAM_KEY_ERROR: &str = "x-error";
pub const CALLBACK_PARAM_KEY_CANCEL: &str = "x-cancel";

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CallbackParams {
    source: Option<String>,
    success: Option<String>,
    error: Option<String>,
    cancel: Option<String>,
}

impl CallbackParams {
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

    pub fn iter(&self) -> CallbackParamsIter {
        let callback_params: Vec<_> = vec![
                (CALLBACK_PARAM_KEY_SOURCE, &self.source),
                (CALLBACK_PARAM_KEY_SUCCESS, &self.success),
                (CALLBACK_PARAM_KEY_ERROR, &self.error),
                (CALLBACK_PARAM_KEY_CANCEL, &self.cancel),
            ]
            .into_iter()
            .filter_map(|(k, v)| v.as_ref().map(|v| (k, v.as_ref())))
            .collect();

        CallbackParamsIter { callback_params }
    }
}

impl<T> FromIterator<(T, T)> for CallbackParams
    where T: ToString
{
    fn from_iter<I: IntoIterator<Item=(T, T)>>(iter: I) -> Self {
        let mut callback_params = CallbackParams::default();

        for (k, v) in iter.into_iter() {
            let key = k.to_string();
            match key.as_ref() {
                CALLBACK_PARAM_KEY_SOURCE => callback_params.set_source(Some(v.to_string())),
                CALLBACK_PARAM_KEY_SUCCESS => callback_params.set_success(Some(v.to_string())),
                CALLBACK_PARAM_KEY_ERROR => callback_params.set_error(Some(v.to_string())),
                CALLBACK_PARAM_KEY_CANCEL => callback_params.set_cancel(Some(v.to_string())),
                _ => {}
            }
        }

        callback_params
    }
}

pub struct CallbackParamsIter<'a> {
    callback_params: Vec<(&'a str, &'a str)>,
}

impl<'a> Iterator for CallbackParamsIter<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        self.callback_params
            .pop()
            .map(|(k, v)| (Cow::Borrowed(k), Cow::Borrowed(v)))
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ActionParams {
    action_params: Vec<(String, String)>,
}

impl ActionParams {
    pub fn clear(&mut self) {
        self.action_params.clear();
    }

    pub fn append<I, K, V>(&mut self, action_params: I)
        where
            I: IntoIterator,
            I::Item: Borrow<(K, V)>,
            K: ToString,
            V: ToString,
    {
        let mut action_params = action_params
            .into_iter()
            .map(|i| {
                let (k, v) = i.borrow();
                (k.to_string(), v.to_string())
            })
            .collect();
        self.action_params.append(&mut action_params);
    }

    pub fn push<K, V>(&mut self, key: K, value: V)
        where
            K: ToString,
            V: ToString,
    {
        self.action_params
            .push((key.to_string(), value.to_string()));
    }

    fn is_callback_param<T: AsRef<str>>(key: T) -> bool {
        key.as_ref().starts_with("x-")
    }

    pub fn iter(&self) -> ActionParamsIter {
        ActionParamsIter {
            action_params: &self.action_params
        }
    }
}

impl<T> FromIterator<(T, T)> for ActionParams
    where T: ToString
{
    fn from_iter<I: IntoIterator<Item=(T, T)>>(iter: I) -> Self {
        let action_params = iter
            .into_iter()
            .filter_map(|(k, v)| {
                let key = k.to_string();

                if !ActionParams::is_callback_param(&key) {
                    Some((key, v.to_string()))
                } else {
                    None
                }
            })
            .collect();

        ActionParams {
            action_params
        }
    }
}

pub struct ActionParamsIter<'a> {
    action_params: &'a [(String, String)],
}

impl<'a> Iterator for ActionParamsIter<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.action_params.first() {
            Some((k, v)) => {
                self.action_params = &self.action_params[1..];
                Some((
                    Cow::Borrowed(k),
                    Cow::Borrowed(v),
                ))
            },
            None => None
        }
    }
}

// impl<'a> IntoIterator for AcionParams {
//     type Item = (Cow<'a, str>, Cow<'a, str>);
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct XCallbackUrl {
    scheme: String,
    action: String,
    action_params: ActionParams,
    callback_params: CallbackParams,
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

        Ok(XCallbackUrl {
            scheme,
            action,
            action_params: ActionParams::from_iter(url.query_pairs()),
            callback_params: CallbackParams::from_iter(url.query_pairs())
        })
    }

    pub fn new(scheme: &str) -> Self {
        XCallbackUrl {
            scheme: scheme.to_string(),
            action: "".to_string(),
            action_params: ActionParams { action_params: vec![] },
            callback_params: CallbackParams {
                source: None,
                success: None,
                error: None,
                cancel: None
            }
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

    pub fn action_params(&self) -> &ActionParams {
        &self.action_params
    }

    pub fn action_params_mut(&mut self) -> &mut ActionParams {
        &mut self.action_params
    }

    pub fn callback_params(&self) -> &CallbackParams {
        &self.callback_params
    }

    pub fn callback_params_mut(&mut self) -> &mut CallbackParams {
        &mut self.callback_params
    }

    pub fn to_url(&self) -> Result<Url, url::ParseError> {
        let mut url = Url::parse(&format!(
            "{scheme}://{host}/{action}",
            host = CALLBACK_HOST,
            scheme = self.scheme,
            action = self.action,
        ))?;

        let query_pairs: Vec<_> = self.action_params
            .iter()
            .chain(self.callback_params.iter())
            .collect();

        if !query_pairs.is_empty() {
            url.query_pairs_mut().extend_pairs(&query_pairs);
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
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                Some("callback://x-callback-success"),
                url.callback_params().success()
            );
            assert_eq!(url.to_string(), input);
        }

        #[test]
        fn test_no_params() {
            let input = "callback://x-callback-url/action";

            let url = XCallbackUrl::parse(input).unwrap();

            assert_eq!("callback", url.scheme());
            assert_eq!("action", url.action());
            assert_eq!(url.action_params().iter().count(), 0);
            assert_eq!(url.callback_params().source(), None);
            assert_eq!(url.callback_params().success(), None);
            assert_eq!(url.callback_params().error(), None);
            assert_eq!(url.callback_params().cancel(), None);
            assert_eq!(url.to_string(), input);
        }

        #[test]
        fn test_no_action_params() {
            let input = "callback://x-callback-url/action\
                ?x-success=callback%3A%2F%2Fx-callback-success";

            let url = XCallbackUrl::parse(input).unwrap();

            assert_eq!("callback", url.scheme());
            assert_eq!("action", url.action());
            assert_eq!(url.action_params().iter().count(), 0);
            assert_eq!(
                Some("callback://x-callback-success"),
                url.callback_params.success()
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
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<_>>()
            );
            assert_eq!(url.callback_params().source(), None);
            assert_eq!(url.callback_params().success(), None);
            assert_eq!(url.callback_params().error(), None);
            assert_eq!(url.callback_params().cancel(), None);
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
    pub action_params: Vec<(String, String)>,
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
