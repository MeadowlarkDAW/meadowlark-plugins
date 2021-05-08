
use tuix::*;

use femtovg::{renderer::OpenGl, Path, Paint, Align, Baseline, Canvas};

use super::graph::Graph;
use super::graph::ControlPoint;
use super::channel_controls::ChannelControls;

use crate::util::*;

use crate::atomic_f64::AtomicF64;

use std::sync::Arc;

use crate::EQEffectParameters;

#[derive(Debug, Clone, PartialEq)]
pub enum EQEvent {
    StartDrag(usize),
    StopDrag(usize),
    MovePoint(usize, f32, f32),
    SetFreq(usize, f32),
    SetGain(usize, f32),
}

pub struct EQUI {
    params: Arc<EQEffectParameters>,
    sample_rate: Arc<AtomicF64>,    
    selected_control: usize,
    control_points: [Entity; 4],
    controls: Entity,
    header: Entity,
    graph: Entity,
    dragging: bool,
}

impl EQUI {
    pub fn new(params: Arc<EQEffectParameters>, sample_rate: Arc<AtomicF64>) -> Self {
        Self {
            params: params.clone(),
            sample_rate: sample_rate.clone(),
            selected_control: 0,
            control_points: [Entity::null(); 4],
            controls: Entity::null(),
            header: Entity::null(),
            graph: Entity::null(),
            dragging: false,
        }
    }
}

impl Widget for EQUI {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        // self.header = Element::new().build(state, entity, |builder| {
        //     builder
        //         .set_width(Stretch(1.0))
        //         .set_height(Pixels(30.0))
        //         .set_background_color(Color::rgb(40,40,40))
        //         .set_child_space(Stretch(1.0))
        //         .set_text("Header")
        // });

        let graph = Graph::new(self.params.clone(), self.sample_rate.clone()).build(state, entity, |builder| {
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

        self.controls = controls;

        let control_point = ControlPoint::new("1")
        .on_press(|_, state, entity| {state.insert_event(Event::new(EQEvent::StartDrag(0)).target(entity));})
        .on_release(|_, state, entity| {state.insert_event(Event::new(EQEvent::StopDrag(0)).target(entity));})
        .on_move(move |knob, state, entity| {
            //state.insert_event(Event::new(EQEvent::MovePoint(0,knob.px,knob.py)).direct(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(0,knob.px,knob.py)).target(controls));

        })
        .build(state, graph, |builder| builder);

        self.control_points[0] = control_point;

        let control_point = ControlPoint::new("2")
        .on_press(|_, state, entity| {state.insert_event(Event::new(EQEvent::StartDrag(1)).target(entity));})
        .on_release(|_, state, entity| {state.insert_event(Event::new(EQEvent::StopDrag(1)).target(entity));})
        .on_move(move |knob, state, entity| {
            //state.insert_event(Event::new(EQEvent::MovePoint(1,knob.px,knob.py)).direct(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(1,knob.px,knob.py)).target(controls));
        })
        .build(state, graph, |builder| builder);

        self.control_points[1] = control_point;

        let control_point = ControlPoint::new("3")
        .on_press(|_, state, entity| {state.insert_event(Event::new(EQEvent::StartDrag(2)).target(entity));})
        .on_release(|_, state, entity| {state.insert_event(Event::new(EQEvent::StopDrag(2)).target(entity));})
        .on_move(move |knob, state, entity| {
            //state.insert_event(Event::new(EQEvent::MovePoint(2,knob.px,knob.py)).direct(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(2,knob.px,knob.py)).target(controls));
        })
        .build(state, graph, |builder| builder);

        self.control_points[2] = control_point;

        let control_point = ControlPoint::new("4")
        .on_press(|_, state, entity| {state.insert_event(Event::new(EQEvent::StartDrag(3)).target(entity));})
        .on_release(|_, state, entity| {state.insert_event(Event::new(EQEvent::StopDrag(3)).target(entity));})
        .on_move(move |knob, state, entity| {
            //state.insert_event(Event::new(EQEvent::MovePoint(3,knob.px,knob.py)).direct(graph));
            state.insert_event(Event::new(EQEvent::MovePoint(3,knob.px,knob.py)).target(controls));
        })
        .build(state, graph, |builder| builder);

        self.control_points[3] = control_point;

        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(eq_event) = event.message.downcast() {
            match eq_event {

                EQEvent::StartDrag(index) => {
                    self.selected_control = *index;
                    self.dragging = true;
                }

                EQEvent::StopDrag(index) => {
                    self.dragging = false;
                }

                EQEvent::MovePoint(index, x, y) => {
                    if self.dragging {

                        let xx = *x - 40.0;
                        let yy = *y - 40.0;

                        let freq = index_to_freq(xx, 1.301030, 4.3013, 720.0);
                        let gain = index_to_amp(yy, 12.0, -12.0, 370.0);

                        self.params.bands[*index].gain.set(gain as f64);
                        self.params.bands[*index].freq.set(freq as f64);
                    }
                    //self.selected_control = *index;
                }

                EQEvent::SetFreq(index, freq) => {

                    let idx = if *index == 5 {
                        self.selected_control
                    } else {
                        *index
                    };

                    let min = 1.301030;
                    let max = 4.3013;
                    let x = freq_to_index(*freq, min, max, 720.0);
                    self.params.bands[idx].freq.set(*freq as f64);
                    self.control_points[idx].set_left(state, Pixels(x + 30.0));
                    event.consume();
                    //state.insert_event(Event::new(EQEvent::SetFreq(*index, *freq)).direct(self.graph));
                }

                EQEvent::SetGain(index, gain) => {

                    let idx = if *index == 5 {
                        self.selected_control
                    } else {
                        *index
                    };

                    let y = amp_to_index(*gain, 12.0, -12.0, 370.0);
                    self.params.bands[idx].gain.set(*gain as f64);
                    self.control_points[idx].set_top(state, Pixels(y + 30.0));
                    event.consume();

                    //state.insert_event(Event::new(EQEvent::SetGain(*index, *gain)).direct(self.graph));
                }

                // EQEvent::SetKind(index, kind) => {
                //     match kind {
                //         Type::PeakingEQ(val) => {
                //             let y = amp_to_index(*val as f32, 12.0, -12.0, 370.0);
                //             self.controls[self.selected_control].set_top(state, Pixels(y + 30.0));

                //             state.insert_event(Event::new(EQEvent::SetGain(*index, *val as f32)).direct(self.graph));
                //         }

                //         _=> {}
                //     }
                // }

                _=> {}
            }
        }
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas<OpenGl>) {
        for (index, band) in self.params.bands.iter().enumerate() {
            //state.insert_event(Event::new(EQEvent::SetGain(index, band.gain.get() as f32)).target(self.controls)); 
            let gain = self.params.bands[index].gain.get();
            let freq = self.params.bands[index].freq.get();

            let min = 1.301030;
            let max = 4.3013;
            let x = freq_to_index(freq as f32, min, max, 720.0);
            let y = amp_to_index(gain as f32, 12.0, -12.0, 370.0);
            if !self.dragging && index == self.selected_control {
                //self.control_points[self.selected_control].set_left(state, Pixels(x + 30.0));
                //self.control_points[self.selected_control].set_top(state, Pixels(y + 30.0));
                
                // Doesn't work for some reason
                //state.data.set_posx(self.control_points[index], x + 30.0);
                //state.data.set_posy(self.control_points[index], y + 30.0);

                state.insert_event(Event::new(EQEvent::SetGain(index, gain as f32)).direct(self.controls));
                state.insert_event(Event::new(EQEvent::SetFreq(index, freq as f32)).direct(self.controls));
            }
            

            // Set the position directly to avoid a frame delay
            
            //self.header.set_text(state, &gain.to_string());
            
        }
    }
}