use std::borrow::BorrowMut;
use std::error::Error;

use async_trait::async_trait;
use cursive::{Cursive, Rect, Vec2, View};
use cursive::align::Align;
use cursive::event::Key;
use cursive::view::Resizable;
use cursive::views::{Dialog, FixedLayout, OnLayoutView, ResizedView, TextContent, TextView};

use crate::cursive_tokio::{BackgroundEventContext, BackgroundEventHandler, ControllerEventContext, TokioCursive, UIController};

mod cursive_tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    TokioCursive::<RecipeApp, BackgroundStates, RecipeUI, UIStates>::new()
      .start()
      .await;

    Ok(())
}


#[derive(Default)]
enum UIStates {
    #[default]
    Start,
    EnterKeyPressed,
}

enum BackgroundStates {
    EnterKeyPressed,
}


struct RecipeUI(ControllerEventContext<BackgroundStates>);

impl From<ControllerEventContext<BackgroundStates>> for RecipeUI {
    fn from(value: ControllerEventContext<BackgroundStates>) -> Self {
        Self(value)
    }
}

impl RecipeUI {

    fn construct_default_ui(&mut self) -> () {
        self.0.siv.add_global_callback(Key::Esc, |s| s.quit());
        self.0.siv.add_global_callback(Key::Enter, |_| self.0.sender.send(BackgroundStates::EnterKeyPressed));

        let main_window = self.0.siv.screen_mut();
        let text_content = TextContent::new("Press <Esc> to quit!");
        main_window.add_transparent_layer(
            Self::construct_status_bar(text_content),
        );
    }

    fn construct_status_bar(text_content: TextContent) -> ResizedView<OnLayoutView<FixedLayout>> {
        OnLayoutView::new(
            FixedLayout::new().child(
                Rect::from_point(Vec2::zero()),
                TextView::new_with_content(text_content.clone())
                    .align(Align::bot_right())//,
                    .full_width(),
            ),
            |layout, size| {
                let rect = Rect::from_size((0, size.y - 1), (size.x, 1));
                layout.set_child_position(0, rect);
                layout.layout(size);
            },
        )
            .full_screen()
    }
}

impl UIController<BackgroundStates, UIStates> for RecipeUI {

    fn borrow_cursive(&mut self) -> &mut Cursive {
        self.0.siv.borrow_mut()
    }

    fn handle_frontend_event(&'static mut self, e: UIStates) -> Result<(), Box<dyn Error>> {
        match e {
            UIStates::Start => self.construct_default_ui(),
            UIStates::EnterKeyPressed => {
                self.0.siv.add_layer(Dialog::text("Well... it worked!"))
            }
        }

        Ok(())
    }
}

struct RecipeApp(BackgroundEventContext<UIStates>);

impl From<BackgroundEventContext<UIStates>> for RecipeApp {

    fn from(value: BackgroundEventContext<UIStates>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl BackgroundEventHandler<BackgroundStates, UIStates> for RecipeApp {

    async fn handle_backend_event(&self, e: BackgroundStates) -> Result<(), Box<dyn Error>> {
        match e {
            BackgroundStates::EnterKeyPressed => self.0.sender.send(UIStates::EnterKeyPressed)
        }

        Ok(())
    }
}
