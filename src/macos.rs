use crate::x_callback_url::*;
use ::macos::appkit::*;
use ::macos::foundation::*;
use ::macos::{impl_objc_class, Id, ObjCClass};
use objc::declare::ClassDecl;
use objc::runtime::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::sync::{mpsc, Once};

const CALLBACK_SCHEME: &str = "callback";
const CALLBACK_SOURCE: &str = "callback";
const CALLBACK_ACTION_SUCCESS: &str = "success";
const CALLBACK_ACTION_ERROR: &str = "error";
const CALLBACK_ACTION_CANCEL: &str = "cancel";
const CALLBACK_PARAM_KEY_CALLBACK_ID: &str = "callback_id";

lazy_static! {
    static ref CALLBACK_URL_BASE: XCallbackUrl = XCallbackUrl::new(CALLBACK_SCHEME);
}

lazy_static! {
    static ref SENDERS: Mutex<HashMap<String, Sender<XCallbackUrl>>> = Mutex::new(HashMap::new());
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
    callback_id: String,
    receiver: Receiver<XCallbackUrl>,
}

impl NSXCallbackClient {
    pub fn new() -> NSXCallbackClient {
        let callback_id = NSXCallbackClient::generate_callback_id();
        let (sender, receiver) = mpsc::channel();
        NSXCallbackClient::store_sender(&callback_id, sender);
        NSXCallbackClient {
            callback_id,
            receiver,
        }
    }

    fn generate_callback_id() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(32).collect()
    }

    fn store_sender(callback_id: &str, sender: Sender<XCallbackUrl>) {
        SENDERS
            .lock()
            .unwrap()
            .insert(callback_id.to_string(), sender);
    }

    fn generate_callback_url(&self, url: &XCallbackUrl) -> XCallbackUrl {
        let mut callback_url = url.clone();
        let callback_params = self.generate_callback_params();
        callback_url.set_callback_params(&callback_params);
        callback_url
    }

    fn generate_callback_params(&self) -> Vec<(String, String)> {
        fn generate_callback_url(action: &str, callback_id: &str) -> String {
            let mut url = CALLBACK_URL_BASE.clone();
            url.set_action(action);
            url.append_action_param(CALLBACK_PARAM_KEY_CALLBACK_ID, callback_id);
            url.to_string()
        }

        vec![
            (
                CALLBACK_PARAM_KEY_SOURCE.to_string(),
                CALLBACK_SOURCE.to_string(),
            ),
            (
                CALLBACK_PARAM_KEY_SUCCESS.to_string(),
                generate_callback_url(CALLBACK_ACTION_SUCCESS, &self.callback_id),
            ),
            (
                CALLBACK_PARAM_KEY_ERROR.to_string(),
                generate_callback_url(CALLBACK_ACTION_ERROR, &self.callback_id),
            ),
            (
                CALLBACK_PARAM_KEY_CANCEL.to_string(),
                generate_callback_url(CALLBACK_ACTION_CANCEL, &self.callback_id),
            ),
        ]
    }

    fn wait_for_response(&self) -> Result<XCallbackResponse, Box<dyn Error>> {
        let callback_url = self.receiver.recv()?;
        NSXCallbackClient::callback_url_to_response(callback_url)
    }

    fn callback_url_to_response(
        callback_url: XCallbackUrl,
    ) -> Result<XCallbackResponse, Box<dyn Error>> {
        let status = match callback_url.action() {
            CALLBACK_ACTION_SUCCESS => XCallbackStatus::Success,
            CALLBACK_ACTION_ERROR => XCallbackStatus::Error,
            CALLBACK_ACTION_CANCEL => XCallbackStatus::Cancel,
            action => return Err(Box::new(XCallbackError::InvalidAction(action.to_string()))),
        };
        let action_params = callback_url
            .action_params()
            .filter(|(k, _)| k != CALLBACK_PARAM_KEY_CALLBACK_ID)
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(XCallbackResponse {
            status,
            action_params,
        })
    }
}

impl Drop for NSXCallbackClient {
    fn drop(&mut self) {
        SENDERS.lock().unwrap().remove(&self.callback_id);
    }
}

impl XCallbackClient for NSXCallbackClient {
    fn execute(&self, url: &XCallbackUrl) -> Result<XCallbackResponse, Box<dyn Error>> {
        let callback_url = self.generate_callback_url(url);
        open(&callback_url);
        self.wait_for_response()
    }
}

pub fn open(url: &XCallbackUrl) {
    NSWorkspace::shared_workspace().open_url(NSURL::from(NSString::from(&url.to_string())))
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
                _this: &mut Object,
                _cmd: Sel,
                event: Id,
                _reply_event: Id,
            ) {
                let url = NSAppleEventDescriptor::from_ptr(event)
                    .and_then(|event| event.url_param_value())
                    .and_then(|url| url.as_str())
                    .and_then(|s| XCallbackUrl::parse(s).ok())
                    .unwrap();
                let callback_id = url
                    .action_params()
                    .find(|(k, _)| k == CALLBACK_PARAM_KEY_CALLBACK_ID)
                    .unwrap()
                    .1
                    .to_string();
                let senders = SENDERS.lock().unwrap();
                let sender = senders.get(&callback_id).unwrap();

                sender.send(url).unwrap();
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
