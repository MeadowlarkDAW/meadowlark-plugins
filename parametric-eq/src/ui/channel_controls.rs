

use tuix::*;

pub struct ChannelControls {

}

impl ChannelControls {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Widget for ChannelControls {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        ValueKnob::new("Freq", 0.0, 0.0, 1.0).build(state, entity, |builder| 
             builder
                .set_width(Pixels(80.0))
                //.set_background_color(Color::red())
        );

        ValueKnob::new("Gain", 0.0, 0.0, 1.0).build(state, entity, |builder| 
            builder
               .set_width(Pixels(80.0))
               //.set_background_color(Color::red())
        );

        ValueKnob::new("Q", 0.0, 0.0, 1.0).build(state, entity, |builder| 
                builder
                .set_width(Pixels(80.0))
                //.set_background_color(Color::red())
        );

        entity
            .set_layout_type(state, LayoutType::Row)
            //.set_background_color(state, Color::yellow())
            .set_child_space(state, Stretch(1.0))
    }
}