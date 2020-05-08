use callback::client::{XCallbackClient, XCallbackResponse, XCallbackStatus};
use callback::macos::{run_app, terminate_app, NSXCallbackClient};
use callback::x_callback_url::*;
use std::thread;
use structopt::StructOpt;

fn main() {
    thread::spawn(move || {
        run(NSXCallbackClient::new());
        terminate_app();
    });
    run_app();
}

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

pub fn run<T: XCallbackClient>(client: T) {
    let opts = CallbackOpts::from_args();
    let execute_url = opts_to_url(&opts);
    let response = client.execute(&execute_url).unwrap();
    print_response(&response);
}

fn opts_to_url(opts: &CallbackOpts) -> XCallbackUrl {
    let mut callback_url = XCallbackUrl::new(&opts.scheme);
    callback_url.set_action(&opts.action);
    callback_url.action_params_mut().append(&opts.parameters);
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
    let status = match response.status {
        XCallbackStatus::Success => "success",
        XCallbackStatus::Error => "error",
        XCallbackStatus::Cancel => "cancel",
    };
    println!("{}", status);

    for (k, v) in &response.action_params {
        if !v.is_empty() {
            println!("{}={}", k, v)
        }
    }
}
