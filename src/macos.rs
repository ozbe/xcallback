use ::macos::appkit::*;
use ::macos::foundation::*;
use ::macos::{impl_objc_class, Id, ObjCClass};
use objc::declare::ClassDecl;
use objc::runtime::*;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Once, mpsc};
use url::Url;
use crate::x_callback_url::*;
use std::error::Error;
use std::collections::HashMap;
use std::sync::Mutex;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Formatter};

const CALLBACK_SCHEME: &str = "callback";
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

lazy_static! {
    static ref SENDERS: Mutex<HashMap<String, Sender<String>>> = Mutex::new(HashMap::new());
}

pub fn run_app() {
    let delegate = AppDelegate::new();
    let app = nsapp();
    app.set_delegate(&delegate);
    app.run();
}

pub fn terminate_app() {
    let app = nsapp();
    app.terminate(&app);
}

pub struct NSXCallbackClient {
    key: String,
    receiver: Receiver<String>,
}

impl NSXCallbackClient {
    pub fn new() -> NSXCallbackClient {
        let key = NSXCallbackClient::generate_key();
        let (sender, receiver) = mpsc::channel();

       NSXCallbackClient::store_sender(&key, sender);

        NSXCallbackClient {
            key,
            receiver,
        }
    }

    fn store_sender(key: &str, sender: Sender<String>) {
        SENDERS.lock()
            .unwrap()
            .insert(key.to_string(), sender);
    }

    fn generate_key() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect()
    }
}

impl XCallbackClient for NSXCallbackClient {
    fn execute(&self, url: &XCallbackUrl) -> Result<XCallbackResponse, Box<dyn Error>> {
        let mut callback_url = url.clone();
        let action_params: Vec<(String, String)> = callback_url
            .action_params()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let callback_parameters = [
            (CALLBACK_PARAM_KEY_SOURCE.to_string(), self.key.clone()),
            (CALLBACK_PARAM_KEY_SUCCESS.to_string(), CALLBACK_URL_SUCCESS.as_str().to_string()),
            (CALLBACK_PARAM_KEY_ERROR.to_string(), CALLBACK_URL_ERROR.as_str().to_string()),
            (CALLBACK_PARAM_KEY_CANCEL.to_string(), CALLBACK_URL_CANCEL.as_str().to_string()),
        ];
        callback_url.set_params(
            action_params.iter().chain(callback_parameters.iter())
        );

        open(&callback_url.to_url());

        let url = self.receiver.recv()?;
        let callback_url = XCallbackUrl::parse(&url).unwrap();

        match callback_url.action() {
            RELATIVE_PATH_SUCCESS => Ok(XCallbackResponse::Success { params: vec![] }),
            RELATIVE_PATH_ERROR => Ok(XCallbackResponse::Error { params: vec![] }),
            RELATIVE_PATH_CANCEL => Ok(XCallbackResponse::Cancel { params: vec![] }),
            action => Err(Box::new((XCallbackError::InvalidAction(action.to_string())))),
        }
    }
}

#[derive(Debug)]
pub enum XCallbackError {
    InvalidAction(String),
}

impl Display for XCallbackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            XCallbackError::InvalidAction(action) => f.write_fmt(format_args!("Invalid action: {}", action))
        }
    }
}

impl Error for XCallbackError {}

fn open(url: &Url) {
    NSWorkspace::shared_workspace().open_url(NSURL::from(NSString::from(url.as_str())))
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

            extern "C" fn app_will_finish_launching(this: &mut Object, _cmd: Sel, _note: Id) {
                if let Some(delegate) = AppDelegate::from_ptr(this) {
                    NSAppleEventManager::shared_manager().set_get_url_event_handler(&delegate);
                }
            }

            extern "C" fn event_handler_handle_get_url(
                _: &mut Object,
                _cmd: Sel,
                event: Id,
                _reply_event: Id,
            ) {
                let url = NSAppleEventDescriptor::from_ptr(event)
                    .and_then(|event| event.url_param_value())
                    .and_then(|url| url.as_str())
                    .and_then(|s| XCallbackUrl::parse(s).ok())
                    .unwrap();

                let senders = SENDERS.lock().unwrap();
                let sender = senders.get(&url.source().unwrap()).unwrap();

                sender.send(url.as_str().to_string()).unwrap();
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
