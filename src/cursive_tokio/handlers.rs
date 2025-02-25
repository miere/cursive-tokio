use std::error::Error;
use async_trait::async_trait;
use cursive::Cursive;

/// Represents the main UI controller. The idea is to provide an abstraction
/// that allows us to handle UI-related events (possibly representing UI states).
/// The target object of this trait will be bound to the foreground (main) thread.
pub trait UIController<BackendEvent, FrontendEvent> {

    // TODO: this is temporary workaround. We might need to revisit this design
    //  and ensure the UI don't have hand back an object that was previously under our control.
    fn borrow_cursive(&mut self) -> &mut Cursive;

    /// Handle UI-related events.
    fn handle_frontend_event(&'static mut self, e: FrontendEvent) -> Result<(), Box<dyn Error>>;
}

/// This is analogous to the `UIController`, providing means for the background
/// job to handle events sent by the UI. It is important to keep in mind that the
/// target object of this trait (self) will be bound to a second thread that will
/// be running in parallel to the main one.
#[async_trait]
pub trait BackgroundEventHandler<BackendEvent, FrontendEvent> {

    /// Handle backend events.
    async fn handle_backend_event(&self, e: BackendEvent) -> Result<(), Box<dyn Error>>;
}