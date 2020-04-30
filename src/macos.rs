use ::macos::appkit::*;
use ::macos::foundation::*;
use ::macos::{impl_objc_class, Id, ObjCClass};
use objc::declare::ClassDecl;
use objc::runtime::*;
use std::sync::mpsc::Sender;
use std::sync::Once;
use url::Url;

static mut SENDER: Option<Sender<String>> = None;

pub fn run_app(sender: Sender<String>) {
    unsafe { SENDER = Some(sender) };
    let delegate = AppDelegate::new();
    let app = nsapp();
    app.set_delegate(&delegate);
    app.run();
}

pub fn terminate_app() {
    let app = nsapp();
    app.terminate(&app);
}

pub fn open(url: &Url) {
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
