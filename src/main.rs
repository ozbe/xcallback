#[macro_use]
extern crate objc;

use macos::appkit::*;
use macos::foundation::*;
use macos::{impl_objc_class, nil, Id, ObjCClass};
use objc::declare::ClassDecl;
use objc::runtime::*;
use std::ops::Deref;
use std::process::{Command, Output};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Once};
use std::thread;
use structopt::StructOpt;

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
    parameters: Vec<String>,
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
    let url = opts_to_url(&opts);

    let _ = execute(&url).unwrap();

    let result = receiver.recv().unwrap();
    println!("{}", result);

    terminate_ns_app();
}

fn run_ns_app() {
    let delegate = AppDelegate::new();
    let app = nsapp();
    app.set_delegate(&delegate);
    app.run();
}

fn terminate_ns_app() {
    let app = nsapp();
    unsafe { msg_send![app.ptr(), terminate: nil] }
}

fn execute(url: &str) -> Result<Output, std::io::Error> {
    Command::new("open").arg(url).output()
}

fn opts_to_url(opts: &CallbackOpts) -> String {
    let callback_parameters = vec![
        format!("x-source={}", "Callback"),
        format!("x-success={}", CALLBACK_ADDR),
        format!("x-error={}", CALLBACK_ADDR),
        format!("x-cancel={}", CALLBACK_ADDR),
    ];

    let parameters: Vec<&str> = opts
        .parameters
        .iter()
        .chain(callback_parameters.iter())
        .map(|p| p.deref())
        .collect();

    format!(
        "{scheme}://{host}/{action}{query}",
        host = HOST,
        scheme = opts.scheme,
        action = opts.action,
        query = format!(
            "{prefix}{parameters}",
            prefix = if parameters.is_empty() { "" } else { "?" },
            parameters = parameters.join("&")
        )
    )
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