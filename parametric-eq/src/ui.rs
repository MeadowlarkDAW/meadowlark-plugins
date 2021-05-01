use rtrb::Consumer;
use raw_window_handle::HasRawWindowHandle;

use tuix::*;

mod graph;
use graph::*;

mod channel_controls;
use channel_controls::*;

static THEME: &str = include_str!("ui/theme.css");

pub enum UIHandleMsg {
    CloseWindow
}


pub fn build_and_run(handle_msg_rx: Consumer<UIHandleMsg>, parent_window: &impl HasRawWindowHandle) {
    // TODO: Make app respond to close events from `handle_msg_rx`.
    let window_description = WindowDescription::new().with_title("EQ PLUGIN").with_inner_size(800, 600);
    let app = Application::new(window_description, |state, window| {
        state.add_theme(THEME);

        let header = Element::new().build(state, window, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Pixels(30.0))
                .set_background_color(Color::rgb(40,40,40))
                .set_child_space(Stretch(1.0))
                .set_text("Header")
        });

        let graph = Graph::new().build(state, window, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Stretch(1.0))
                .set_background_color(Color::rgb(30,30,30))
                .set_child_space(Stretch(1.0))
                .set_text("Graph")
        });

        let control_point = ControlPoint::new().build(state, graph, |builder| builder);

        let controls = Element::new().build(state, window, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Pixels(150.0))
                .set_background_color(Color::rgb(80,80,80))
                .set_child_left(Stretch(1.0))
                .set_child_right(Stretch(1.0))
                .set_text("Controls")
        });

        ChannelControls::new().build(state, controls, |builder|
            builder
                .set_width(Units::Auto)
                .set_height(Stretch(1.0))
        );
    });

    app.open_parented(parent_window);
}