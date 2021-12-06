use tuix::*;

use super::EQEvent;

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
            .on_changing(|knob, state, entity| {
                state.insert_event(Event::new(EQEvent::SetGain(5, knob.value)).target(entity));
            })
            .build(
                state,
                entity,
                |builder| {
                    builder
                        .set_width(Pixels(80.0))
                        .set_left(Pixels(20.0))
                        .set_right(Pixels(10.0))
                }, //.set_background_color(Color::red())
            );

        self.freq_knob = ValueKnob::new("FREQ", 1000.0, 20.0, 20000.0)
            .with_log_scale()
            .with_units(UnitsType::Hertz)
            .on_changing(|knob, state, entity| {
                state.insert_event(Event::new(EQEvent::SetFreq(5, knob.value)).target(entity));
            })
            .build(
                state,
                entity,
                |builder| {
                    builder
                        .set_width(Pixels(80.0))
                        .set_left(Pixels(10.0))
                        .set_right(Pixels(10.0))
                }, //.set_background_color(Color::red())
            );

        // Buttons
        let row = Row::new().build(state, entity, |builder| {
            builder
                .set_width(Pixels(90.0))
                .set_height(Pixels(90.0))
                .set_left(Pixels(10.0))
                .set_right(Pixels(10.0))
        });

        let left_col = Column::new().build(state, row, |builder| {
            builder
                .set_width(Pixels(30.0))
                .set_left(Pixels(0.0))
                .set_right(Pixels(0.0))
        });

        Button::with_label("A")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 1.0));
            })
            .build(state, left_col, |builder| {
                builder.set_background_color(Color::rgb(100, 50, 50))
            });
        Button::with_label("D")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 2.0));
            })
            .build(state, left_col, |builder| {
                builder.set_background_color(Color::rgb(0, 100, 50))
            });
        Button::with_label("G")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 3.0));
            })
            .build(state, left_col, |builder| {
                builder.set_background_color(Color::rgb(0, 50, 100))
            });

        let middle_col = Column::new().build(state, row, |builder| {
            builder
                .set_width(Pixels(30.0))
                .set_left(Pixels(0.0))
                .set_right(Pixels(0.0))
        });

        Button::with_label("B")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 4.0));
            })
            .build(state, middle_col, |builder| {
                builder.set_background_color(Color::rgb(0, 100, 50))
            });
        Button::with_label("E")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 5.0));
            })
            .build(state, middle_col, |builder| {
                builder.set_background_color(Color::rgb(0, 50, 100))
            });
        Button::with_label("H")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 6.0));
            })
            .build(state, middle_col, |builder| {
                builder.set_background_color(Color::rgb(100, 50, 50))
            });

        let right_col = Column::new().build(state, row, |builder| {
            builder
                .set_width(Pixels(30.0))
                .set_left(Pixels(0.0))
                .set_right(Pixels(0.0))
        });

        Button::with_label("C")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 7.0));
            })
            .build(state, right_col, |builder| {
                builder.set_background_color(Color::rgb(0, 50, 100))
            });
        Button::with_label("F")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 8.0));
            })
            .build(state, right_col, |builder| {
                builder.set_background_color(Color::rgb(100, 50, 50))
            });
        Button::with_label("I")
            .on_press(|_, state, button| {
                button.emit(state, EQEvent::SetKind(5, 9.0));
            })
            .build(state, right_col, |builder| {
                builder.set_background_color(Color::rgb(0, 100, 50))
            });

        ValueKnob::new("WIDTH", 1.0, 0.1, 24.0)
            .with_units(UnitsType::dB)
            .on_changing(|knob, state, entity| {
                state.insert_event(Event::new(EQEvent::SetWidth(5, knob.value)).target(entity));
            })
            .build(
                state,
                entity,
                |builder| {
                    builder
                        .set_width(Pixels(80.0))
                        .set_left(Pixels(10.0))
                        .set_right(Pixels(10.0))
                }, //.set_background_color(Color::red())
            );

        ValueKnob::new("SLOPE", 0.0, 0.0, 16.0)
            .with_units(UnitsType::dB)
            .on_changing(|knob, state, entity| {
                state.insert_event(Event::new(EQEvent::SetSlope(5, knob.value)).target(entity));
            })
            .build(
                state,
                entity,
                |builder| {
                    builder
                        .set_width(Pixels(80.0))
                        .set_left(Pixels(10.0))
                        .set_right(Pixels(20.0))
                }, //.set_background_color(Color::red())
            );

        entity
            .set_layout_type(state, LayoutType::Row)
            //.set_background_color(state, Color::yellow())
            .set_child_space(state, Stretch(1.0))
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(eq_event) = event.message.downcast() {
            match eq_event {
                EQEvent::MovePoint(_, x, y) => {
                    let xx = *x - 40.0;
                    let yy = *y - 40.0;

                    let freq = index_to_freq(xx, 1.301030, 4.3013, 720.0);
                    let amp = index_to_amp(yy, 12.0, -12.0, 370.0);

                    state.insert_event(
                        Event::new(SliderEvent::SetValue(amp)).target(self.gain_knob),
                    );
                    state.insert_event(
                        Event::new(SliderEvent::SetValue(freq)).target(self.freq_knob),
                    );
                }

                EQEvent::SetGain(index, gain) => {
                    state.insert_event(
                        Event::new(SliderEvent::SetValue(*gain)).target(self.gain_knob),
                    );
                }

                EQEvent::SetFreq(index, freq) => {
                    state.insert_event(
                        Event::new(SliderEvent::SetValue(*freq)).target(self.freq_knob),
                    );
                }

                _ => {}
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
