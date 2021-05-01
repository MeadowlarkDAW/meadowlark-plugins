use rtrb::Consumer;
use raw_window_handle::HasRawWindowHandle;

use tuix::*;

pub enum UIHandleMsg {
    CloseWindow
}


pub fn build_and_run(handle_msg_rx: Consumer<UIHandleMsg>, parent_window: &impl HasRawWindowHandle) {
    // TODO: Make app respond to close events from `handle_msg_rx`.

    let app = Application::new(|win_desc, state, window| {
        Button::with_label("Button").build(state, window, |builder| {
            builder
                .set_width(Length::Pixels(100.0))
                .set_height(Length::Pixels(30.0))
                .set_background_color(Color::from("#ff5e1a"))
                .set_text_justify(Justify::Center)
        });

        win_desc.with_title("Hello GUI")
    });

    app.open_parented(parent_window);
}