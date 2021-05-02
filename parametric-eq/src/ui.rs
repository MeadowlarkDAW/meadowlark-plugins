use baseplug::Param;
use rtrb::Consumer;
use raw_window_handle::HasRawWindowHandle;
use triple_buffer::{Input, Output, TripleBuffer};
use atomic_refcell::AtomicRefCell;
use tuix::*;

mod graph;
use graph::*;

mod channel_controls;
use channel_controls::*;

use std::sync::{Arc, Mutex};
use std::{cell::UnsafeCell, rc::Rc};

use super::ParametricEQShared;

static THEME: &str = include_str!("ui/theme.css");

pub enum UIHandleMsg {
    CloseWindow
}

struct ParametricEQUI {
    pub consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>,
}

impl ParametricEQUI {
    fn new(consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>) -> Self {
        Self {
            consumer,
        }
    }
}

impl Widget for ParametricEQUI {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let header = Element::new().build(state, entity, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Pixels(30.0))
                .set_background_color(Color::rgb(40,40,40))
                .set_child_space(Stretch(1.0))
                .set_text("Header")
        });

        let graph = Graph::new(self.consumer.clone()).build(state, entity, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Stretch(1.0))
                .set_background_color(Color::rgb(30,30,30))
                .set_child_space(Stretch(1.0))
                .set_text("Graph")
        });

        // let control_point = ControlPoint::new().build(state, graph, |builder| builder);

        // let controls = Element::new().build(state, entity, |builder| {
        //     builder
        //         .set_width(Stretch(1.0))
        //         .set_height(Pixels(150.0))
        //         .set_background_color(Color::rgb(80,80,80))
        //         .set_child_left(Stretch(1.0))
        //         .set_child_right(Stretch(1.0))
        //         .set_text("Controls")
        // });

        // ChannelControls::new().build(state, controls, |builder|
        //     builder
        //         .set_width(Units::Auto)
        //         .set_height(Stretch(1.0))
        // );

        entity
    }
}


pub fn build_and_run(handle_msg_rx: Consumer<UIHandleMsg>, parent_window: &impl HasRawWindowHandle, shared: &ParametricEQShared) {

    let consumer = shared.consumer.clone();

    let window_description = WindowDescription::new().with_title("EQ PLUGIN").with_inner_size(800, 600);
    let app = Application::new(window_description, move |state, window| {
        state.add_theme(THEME);

        ParametricEQUI::new(consumer.clone()).build(state, window, |builder| builder);
    });

    app.open_parented(parent_window);
}