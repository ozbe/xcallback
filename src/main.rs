#[macro_use]
extern crate objc;

use macos::appkit::*;
use macos::foundation::*;
use macos::{impl_objc_class, Id, ObjCClass};
use objc::declare::ClassDecl;
use objc::runtime::*;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Once};
use std::thread;
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
const CALLBACK_ADDR: &str = "callback://x-callback-url";

static mut SENDER: Option<Sender<String>> = None;

fn main() {
    thread::spawn(cli);
    run_ns_app();
}

fn cli() {
    let (sender, receiver) = mpsc::channel();
    unsafe { SENDER = Some(sender) };

    let opts = CallbackOpts::from_args();
    let execute_url = opts_to_url(&opts);
    execute(&execute_url);

    let result = receiver.recv().unwrap();
    let callback_url = Url::parse(&result).unwrap();
    print_parameters(&callback_url);

    terminate_ns_app();
}

fn print_parameters(url: &Url) {
    if let Some(query) = url.query() {
        for parameter in query.split("&") {
            if !parameter.is_empty() {
                println!("{}", parameter)
            }
        }
    }
}

fn run_ns_app() {
    let delegate = AppDelegate::new();
    let app = nsapp();
    app.set_delegate(&delegate);
    app.run();
}

fn terminate_ns_app() {
    let app = nsapp();
    app.terminate(&app);
}

fn execute(url: &Url) {
    NSWorkspace::shared_workspace()
        .open_url(NSURL::from(NSString::from(url.as_str())))
}

fn opts_to_url(opts: &CallbackOpts) -> Url {
    let mut url = Url::parse(&format!(
        "{scheme}://{host}/{action}",
        scheme = opts.scheme,
        host = HOST,
        action = opts.action,
    )).unwrap();

    let callback_parameters = vec![
        ("x-source", "Callback"),
        ("x-success", CALLBACK_ADDR),
        ("x-error", CALLBACK_ADDR),
        ("x-cancel", CALLBACK_ADDR),
    ];

    url.query_pairs_mut()
        .extend_pairs(&opts.parameters)
        .extend_pairs(&callback_parameters);
    url
}

fn parse_parameter(src: &str) -> Result<(String, String), String> {
    let split: Vec<&str> = src.split("=").collect();
    match split[..] {
        [first, second] => Ok((first.to_string(), second.to_string())),
        _ => Err("Invalid parameter format".to_string())
    }
}

impl_objc_class!(AppDelegate);

impl AppDelegate {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for AppDelegate {
    fn default() -> Self {
        static REGISTER_CUSTOM_CLASS: Once = Once::new();

        REGISTER_CUSTOM_CLASS.call_once(|| {
            let mut decl = ClassDecl::new(AppDelegate::class_name(), class!(NSObject)).unwrap();

            extern fn app_will_finish_launching(this: &mut Object, _cmd: Sel, _note: Id) {
                if let Some(delegate) = AppDelegate::from_ptr(this) {
                    NSAppleEventManager::shared_manager().set_get_url_event_handler(&delegate);
                }
            }

            extern fn event_handler_handle_get_url(
                _: &mut Object,
                _cmd: Sel,
                event: Id,
                _reply_event: Id,
            ) {
                let url = NSAppleEventDescriptor::from_ptr(event)
                    .and_then(|event| event.url_param_value())
                    .and_then(|url| url.as_str());
                let sender = unsafe { SENDER.as_ref().clone().unwrap() };
                sender.send(url.unwrap().to_string()).unwrap();
            }

            unsafe {
                let application_will_finish_launching: extern "C" fn(&mut Object, Sel, Id) =
                    app_will_finish_launching;
                decl.add_method(
                    sel!(applicationWillFinishLaunching:),
                    application_will_finish_launching,
                );

                let handle_get_url: extern "C" fn(&mut Object, Sel, Id, Id) =
                    event_handler_handle_get_url;
                decl.add_method(sel!(handleGetURLEvent:withReplyEvent:), handle_get_url);
            }

            decl.register();
        });

        AppDelegate {
            ptr: unsafe { msg_send![class!(AppDelegate), new] },
        }
    }
}