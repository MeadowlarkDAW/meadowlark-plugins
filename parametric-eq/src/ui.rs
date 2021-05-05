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
use crate::util::*;

static THEME: &str = include_str!("ui/theme.css");

#[derive(Debug, Clone, PartialEq)]
pub enum EQEvent {
    MovePoint(usize, f32, f32),
    SetFreq(usize, f32),
    SetGain(usize, f32),
}

pub enum UIHandleMsg {
    CloseWindow
}

pub struct ParametricEQUI {
    pub consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>,
    selected_control: usize,
    controls: [Entity; 3],
    graph: Entity,
}

impl ParametricEQUI {
    pub fn new(consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>) -> Self {
        Self {
            consumer,
            selected_control: 0,
            controls: [Entity::null(); 3],
            graph: Entity::null(),
        }
    }
}

impl Widget for ParametricEQUI {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        // let header = Element::new().build(state, entity, |builder| {
        //     builder
        //         .set_width(Stretch(1.0))
        //         .set_height(Pixels(30.0))
        //         .set_background_color(Color::rgb(40,40,40))
        //         .set_child_space(Stretch(1.0))
        //         .set_text("Header")
        // });

        let graph = Graph::new(self.consumer.clone()).build(state, entity, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Stretch(1.0))
                .set_background_color(Color::rgb(33,30,33))
                .set_child_space(Stretch(1.0))
                .set_text("Graph")
        });

        self.graph = graph;

        let controls = Element::new().build(state, entity, |builder| {
            builder
                .set_width(Stretch(1.0))
                .set_height(Pixels(150.0))
                .set_background_color(Color::rgb(33,30,33))
                .set_child_left(Stretch(1.0))
                .set_child_right(Stretch(1.0))
        });

        let controls = ChannelControls::new()
        .build(state, controls, |builder|
            builder
                .set_width(Units::Auto)
                .set_height(Stretch(1.0))
        );

        let control_point = ControlPoint::new("1")
        .on_move(move |knob, state, entity| {
            state.insert_event(Event::new(EQEvent::MovePoint(0,knob.px,knob.py)).target(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(0,knob.px,knob.py)).target(controls));

        })
        .build(state, graph, |builder| builder);

        self.controls[0] = control_point;

        let control_point = ControlPoint::new("2")
        .on_move(move |knob, state, entity| {
            state.insert_event(Event::new(EQEvent::MovePoint(1,knob.px,knob.py)).target(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(1,knob.px,knob.py)).target(controls));
        })
        .build(state, graph, |builder| builder);

        self.controls[1] = control_point;

        let control_point = ControlPoint::new("3")
        .on_move(move |knob, state, entity| {
            state.insert_event(Event::new(EQEvent::MovePoint(2,knob.px,knob.py)).target(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(2,knob.px,knob.py)).target(controls));
        })
        .build(state, graph, |builder| builder);

        self.controls[2] = control_point;

        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(eq_event) = event.message.downcast() {
            match eq_event {

                EQEvent::MovePoint(index, _, _) => {
                    self.selected_control = *index;
                }

                EQEvent::SetFreq(index, freq) => {

                    let min = 1.477121;
                    let max = 4.3013;
                    let x = freq_to_index(*freq, min, max, 720.0);

                    self.controls[self.selected_control].set_left(state, Pixels(x + 30.0));

                    state.insert_event(Event::new(EQEvent::SetFreq(self.selected_control, *freq)).direct(self.graph));
                }

                EQEvent::SetGain(index, gain) => {

                    let min = 1.477121;
                    let max = 4.3013;
                    let x = amp_to_index(*gain, 12.0, -12.0, 370.0);

                    self.controls[self.selected_control].set_top(state, Pixels(x + 30.0));

                    state.insert_event(Event::new(EQEvent::SetGain(self.selected_control, *gain)).direct(self.graph));
                }

                _=> {}
            }
        }
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