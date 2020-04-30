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

const HOST: &str = "x-callback-url";
const CALLBACK_SOURCE: &str = "callback";
const RELATIVE_PATH_SUCCESS: &str = "success";
const RELATIVE_PATH_ERROR: &str = "error";
const RELATIVE_PATH_CANCEL: &str = "cancel";
const CALLBACK_PARAM_KEY_SOURCE: &str = "x-source";
const CALLBACK_PARAM_KEY_SUCCESS: &str = "x-success";
const CALLBACK_PARAM_KEY_ERROR: &str = "x-error";
const CALLBACK_PARAM_KEY_CANCEL: &str = "x-cancel";

lazy_static! {
    static ref CALLBACK_URL_BASE: Url = {
        // The stand-in `action` in the path serves to avoid a problem where Url
        // parses the Url successfully, but when a path is set later, the Url
        // serialization is missing the '/` between the host and path.
        let mut url = Url::parse("callback://x-callback-url/action").unwrap();
        url.set_path("");
        url
    };
    static ref CALLBACK_URL_SUCCESS: Url =
        { CALLBACK_URL_BASE.join(RELATIVE_PATH_SUCCESS).unwrap() };
    static ref CALLBACK_URL_ERROR: Url = { CALLBACK_URL_BASE.join(RELATIVE_PATH_ERROR).unwrap() };
    static ref CALLBACK_URL_CANCEL: Url = { CALLBACK_URL_BASE.join(RELATIVE_PATH_CANCEL).unwrap() };
}

pub fn run(receiver: Receiver<String>, execute: &dyn Fn(&Url) -> ()) {
    let opts = CallbackOpts::from_args();
    let execute_url = opts_to_url(&opts);
    execute(&execute_url);

    let result = receiver.recv().unwrap();
    let callback_url = Url::parse(&result).unwrap();
    print_url(&callback_url);
}

fn opts_to_url(opts: &CallbackOpts) -> Url {
    let mut url = Url::parse(&format!(
        "{scheme}://{host}/{action}",
        scheme = opts.scheme,
        host = HOST,
        action = opts.action,
    ))
    .unwrap();

    let callback_parameters = vec![
        (CALLBACK_PARAM_KEY_SOURCE, CALLBACK_SOURCE),
        (CALLBACK_PARAM_KEY_SUCCESS, CALLBACK_URL_SUCCESS.as_str()),
        (CALLBACK_PARAM_KEY_ERROR, CALLBACK_URL_ERROR.as_str()),
        (CALLBACK_PARAM_KEY_CANCEL, CALLBACK_URL_CANCEL.as_str()),
    ];

    url.query_pairs_mut()
        .extend_pairs(&opts.parameters)
        .extend_pairs(&callback_parameters);
    url
}

fn parse_parameter(src: &str) -> Result<(String, String), String> {
    let split: Vec<&str> = src.split('=').collect();
    match split[..] {
        [first, second] => Ok((first.to_string(), second.to_string())),
        _ => Err("Invalid parameter format".to_string()),
    }
}

fn print_url(url: &Url) {
    println!("{}", url.path().trim_start_matches('/'));

    if let Some(query) = url.query() {
        for parameter in query.split('&') {
            if !parameter.is_empty() {
                println!("{}", parameter)
            }
        }
    }
}
