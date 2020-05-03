use crate::x_callback_url::*;
use std::sync::mpsc::Receiver;
use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
/// Interact with x-callback-url APIs
///
/// A utility for interacting with local macOS applications using x-callback-url (http://x-callback-url.com).
struct CallbackOpts {
    /// Scheme of target app
    ///
    /// Unique string identifier of the target app.
    ///
    /// Example: bear
    scheme: String,
    /// Name of action
    ///
    /// Action for target app to execute.
    ///
    /// Example: create
    action: String,
    /// x-callback and action parameters
    ///
    /// Space delimited URL encoded x-callback-url parameters
    ///
    /// Example: title=My%20Note%20Title text=First%20line
    #[structopt(parse(try_from_str = parse_parameter))]
    parameters: Vec<(String, String)>,
}

const CALLBACK_SCHEME: &str = "callback";
const CALLBACK_SOURCE: &str = "callback";
const RELATIVE_PATH_SUCCESS: &str = "success";
const RELATIVE_PATH_ERROR: &str = "error";
const RELATIVE_PATH_CANCEL: &str = "cancel";

lazy_static! {
    static ref CALLBACK_URL_BASE: XCallbackUrl = { XCallbackUrl::new(CALLBACK_SCHEME) };
    static ref CALLBACK_URL_SUCCESS: XCallbackUrl = {
        let mut callback_url = CALLBACK_URL_BASE.clone();
        callback_url.set_action(RELATIVE_PATH_SUCCESS);
        callback_url
    };
    static ref CALLBACK_URL_ERROR: XCallbackUrl = {
        let mut callback_url = CALLBACK_URL_BASE.clone();
        callback_url.set_action(RELATIVE_PATH_ERROR);
        callback_url
    };
    static ref CALLBACK_URL_CANCEL: XCallbackUrl = {
        let mut callback_url = CALLBACK_URL_BASE.clone();
        callback_url.set_action(RELATIVE_PATH_CANCEL);
        callback_url
    };
}

pub fn run(client: &dyn XCallbackClient) {
    let opts = CallbackOpts::from_args();
    let execute_url = opts_to_url(&opts);
    let response = client.execute(&execute_url).unwrap();
    print_response(&response);
}

fn opts_to_url(opts: &CallbackOpts) -> XCallbackUrl {
    let mut callback_url = XCallbackUrl::new(&opts.scheme);
    callback_url.set_action(&opts.action);
    let callback_parameters = [
        (CALLBACK_PARAM_KEY_SOURCE, CALLBACK_SOURCE),
        (CALLBACK_PARAM_KEY_SUCCESS, CALLBACK_URL_SUCCESS.as_str()),
        (CALLBACK_PARAM_KEY_ERROR, CALLBACK_URL_ERROR.as_str()),
        (CALLBACK_PARAM_KEY_CANCEL, CALLBACK_URL_CANCEL.as_str()),
    ];
    let action_params: Vec<(&str, &str)> = opts
        .parameters
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();

    callback_url.set_params(action_params.iter().chain(callback_parameters.iter()));
    callback_url
}

fn parse_parameter(src: &str) -> Result<(String, String), String> {
    let split: Vec<&str> = src.split('=').collect();
    match split[..] {
        [first, second] => Ok((first.to_string(), second.to_string())),
        _ => Err("Invalid parameter format".to_string()),
    }
}

fn print_response(response: &XCallbackResponse) {
    let params = match response {
        XCallbackResponse::Success { params } => params,
        XCallbackResponse::Error { params } => params,
        XCallbackResponse::Cancel { params } => params,
    };

    // println!("{}", response.);

    for (k, v) in params {
        if !v.is_empty() {
            println!("{}={}", k, v)
        }
    }
}
