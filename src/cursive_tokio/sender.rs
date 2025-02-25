use std::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;

pub struct BackgroundEventSender<BackendEvent>(pub UnboundedSender<BackendEvent>);

impl<BackendEvent> BackgroundEventSender<BackendEvent> {

    pub fn send(&self, e: BackendEvent) {
        self.0.send(e).unwrap()
    }
}

pub struct ForegroundEventSender<FrontendEvent>(pub Sender<FrontendEvent>);

impl<FrontendEvent> ForegroundEventSender<FrontendEvent> {

    pub fn send(&self, e: FrontendEvent) {
        self.0.send(e).unwrap()
    }
}