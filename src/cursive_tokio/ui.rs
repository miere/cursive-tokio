use std::marker::PhantomData;
use std::sync::mpsc::Receiver;

use cursive::{backends, Cursive};
use cursive::backend::Backend;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::cursive_tokio::{BackgroundEventHandler, UIController};
use crate::cursive_tokio::{BackgroundEventSender, ForegroundEventSender};

/// Allows Cursive to emit signals to an async, tokio managed, event loop.
///
/// Remember this is just a thin, and very opinionated, layer of abstraction between Cursive
/// and Tokio. It aims to provide means to quickly bootstrap a Cursive app with basic async
/// support. It has no intention to replace Cursive by any means.
pub struct TokioCursive<Background, BackgroundEvent, Controller, ForegroundEvent>
{
    background: Option<Background>,
    background_receiver: Option<UnboundedReceiver<BackgroundEvent>>,
    controller: Option<Controller>,
    controller_receiver: Option<Receiver<ForegroundEvent>>,

    // fields copied from CursiveRunner
    running: bool,

    // See: https://github.com/rust-lang/rfcs/blob/master/text/0738-variance.md#phantom-data
    _phantom: PhantomData<(BackgroundEvent, ForegroundEvent)>
}

impl<Background, BackgroundEvent, Controller, ForegroundEvent>
    TokioCursive<Background, BackgroundEvent, Controller, ForegroundEvent>
where
    ForegroundEvent: Default + Sync + Send + 'static,
    BackgroundEvent: Sync + Send + 'static,

    Background: BackgroundEventHandler<BackgroundEvent, ForegroundEvent> + Sync + Send + 'static,
    Background: From<BackgroundEventContext<ForegroundEvent>>,

    Controller: UIController<BackgroundEvent, ForegroundEvent> + 'static,
    Controller: From<ControllerEventContext<BackgroundEvent>>,
{

    pub fn new() -> Self
    {
        let (async_sender, async_receiver) =  tokio::sync::mpsc::unbounded_channel::<BackgroundEvent>();
        let (sync_sender, sync_receiver) = std::sync::mpsc::channel::<ForegroundEvent>();

        let background_context = BackgroundEventContext {
            sender: ForegroundEventSender(sync_sender),
        };

        let background = Background::from(background_context);

        let controller_context = ControllerEventContext {
            sender: BackgroundEventSender(async_sender),
            siv: Default::default(),
        };
        let controller = Controller::from(controller_context);

        TokioCursive {
            background: Some(background),
            background_receiver: Some(async_receiver),
            controller: Some(controller),
            controller_receiver: Some(sync_receiver),

            running: true,

            _phantom: PhantomData{}
        }
    }

    pub async fn start(mut self)
    {
        self.try_handle_backend_events_in_background().await;
        self.try_handle_frontend_events();
    }

    async fn try_handle_backend_events_in_background(
        &mut self,
    ) {
        // TODO: this is just defensive programing as an attempt to de-couple the variable
        //  content and allow the loop to hold ownership of them. Ideally, we should avoid
        //  this design, employing some proper Rust tricks to make this more elegant.
        let background = self.background.take();
        let receiver = self.background_receiver.take();

        match (background, receiver) {
            (None, None) |
            (None, Some(_)) |
            (Some(_), None) => self.panic_due_to_multiple_instances(),
            (Some(backend), Some(async_receiver)) => {
                Self::handle_backend_events_in_background(backend, async_receiver);
            }
        }
    }

    fn handle_backend_events_in_background(backend: Background, mut async_receiver: UnboundedReceiver<BackgroundEvent>) {
        tokio::spawn(async move {
            loop {
                if let Some(received) = async_receiver.recv().await {
                    backend.handle_backend_event(received).await
                        // TODO: this was created for the sake of experimentation. we shall avoid panics.
                        .expect("Unexpectedly failed to handle backend event");
                }
            }
        });
    }

    fn try_handle_frontend_events(&mut self)
    {
        // TODO: this is just defensive programing as an attempt to de-couple the variable
        //  content and allow the loop to hold ownership of them. Ideally, we should avoid
        //  this design, employing some proper Rust tricks to make this more elegant.
        let controller = self.controller.take();
        let receiver = self.controller_receiver.take();

        match (controller, receiver) {
            (None, None) |
            (None, Some(_)) |
            (Some(_), None) => self.panic_due_to_multiple_instances(),
            (Some(mut controller), Some(receiver)) => self.handle_frontend_events(&mut controller, receiver)
        }
    }

    fn handle_frontend_events(&mut self, controller: &mut Controller, receiver: Receiver<ForegroundEvent>) {
        let siv = controller.borrow_cursive();
        let backend_init = Box::new(backends::try_default);
        let backend: Box<dyn Backend> = backend_init().unwrap();
        let mut runner = siv.runner(backend);

        runner.refresh();

        loop {
            let received_something = runner.process_events();
            runner.post_events(received_something);
            if let Ok(received) = receiver.try_recv() {
                controller.handle_frontend_event(received).unwrap();
            }
            if !self.running {
                break
            }
        }
    }

    // TODO: this was created for the sake of experimentation. we shall avoid panics and use results instead.
    fn panic_due_to_multiple_instances(&self) {
        panic!("Instantiated more than once. Please submit a bug report.")
    }
}

pub struct ControllerEventContext<BackgroundEvent> {
    pub sender: BackgroundEventSender<BackgroundEvent>,
    pub siv: Cursive,
}

pub struct BackgroundEventContext<ForegroundEvent> {
    pub sender: ForegroundEventSender<ForegroundEvent>,
}

