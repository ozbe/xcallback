use crate::x_callback_url::XCallbackUrl;
use std::error::Error;

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
