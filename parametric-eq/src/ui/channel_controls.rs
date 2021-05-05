

use tuix::*;

use crate::EQEvent;

pub struct ChannelControls {
    gain_knob: Entity,
    freq_knob: Entity,

}

impl ChannelControls {
    pub fn new() -> Self {
        Self {
            gain_knob: Entity::null(),
            freq_knob: Entity::null(),
        }
    }
}

impl Widget for ChannelControls {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        self.gain_knob = ValueKnob::new("GAIN", 0.0, -12.0, 12.0)
        .with_units(UnitsType::dB)
        .on_changing(|knob, state, entity|{
            state.insert_event(Event::new(EQEvent::SetGain(0, knob.value)).target(entity));
        })
        .build(state, entity, |builder| 
             builder
                .set_width(Pixels(80.0))
                .set_left(Pixels(30.0))
                .set_right(Pixels(30.0))
                //.set_background_color(Color::red())
        );

        self.freq_knob = ValueKnob::new("FREQ", 30.0, 30.0, 20000.0)
        .with_log_scale()
        .with_units(UnitsType::Hertz)
        .on_changing(|knob, state, entity|{
            state.insert_event(Event::new(EQEvent::SetFreq(0, knob.value)).target(entity));
        })
        .build(state, entity, |builder| 
            builder
                .set_width(Pixels(80.0))
                .set_left(Pixels(30.0))
                .set_right(Pixels(30.0))
               //.set_background_color(Color::red())
        );

        Element::new().build(state, entity, |builder|
            builder
                .set_width(Pixels(100.0))
                .set_left(Pixels(30.0))
                .set_right(Pixels(30.0))
        );

        ValueKnob::new("WIDTH", 0.0, 0.0, 1.0).build(state, entity, |builder| 
                builder
                .set_width(Pixels(80.0))
                .set_left(Pixels(30.0))
                .set_right(Pixels(30.0))
                //.set_background_color(Color::red())
        );

        ValueKnob::new("SLOPE", 0.0, 0.0, 1.0).build(state, entity, |builder| 
            builder
            .set_width(Pixels(80.0))
            .set_left(Pixels(30.0))
            .set_right(Pixels(30.0))
            //.set_background_color(Color::red())
        );

        entity
            .set_layout_type(state, LayoutType::Row)
            //.set_background_color(state, Color::yellow())
            .set_child_space(state, Stretch(1.0))

    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(eq_event) = event.message.downcast() {
            match eq_event {
                EQEvent::MovePoint(_,x,y) => {
                    let xx = *x - 40.0;
                    let yy = *y - 40.0;

                    let freq = index_to_freq(xx, 1.477121, 4.3013, 720.0);
                    let amp = index_to_amp(yy, 12.0, -12.0, 370.0);

                    state.insert_event(Event::new(SliderEvent::SetValue(amp)).target(self.gain_knob));
                    state.insert_event(Event::new(SliderEvent::SetValue(freq)).target(self.freq_knob));
                }

                _=> {}
            }
        }
    }
}

fn index_to_freq(i: f32, min: f32, max: f32, length: f32) -> f32 {
    return 10.0f32.powf(min + (i * (max - min) / length));
}

fn index_to_amp(i: f32, min: f32, max: f32, length: f32) -> f32 {
    return min + (i * (max - min) / length);
}

