

use rtrb::{Consumer, Producer};
use triple_buffer::{Input, Output, TripleBuffer};
use atomic_refcell::AtomicRefCell;
use tuix::*;
use femtovg::{renderer::OpenGl, Path, Paint, Align, Baseline, Canvas};
use rustfft::{Fft, FftPlanner, num_complex::Complex, num_traits::real};

use std::{cell::UnsafeCell, sync::Arc};
use std::cmp::Ordering;

const frequencies: [f32; 27] = [1.477121, 1.60206, 1.69897, 1.778151, 1.845098, 1.90309, 1.954243, 2.0, 2.30103, 2.477121, 2.60206, 2.69897, 2.778151, 2.845098, 2.90309, 2.954243, 3.0, 3.30103, 3.477121, 3.60206, 3.69897, 3.778151, 3.845098, 3.90309, 3.954243, 4.0, 4.30103];

pub struct Graph {
    pub consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>,
    prev_frame: Vec<f32>,
}

impl Graph {
    pub fn new(consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>,) -> Self {
        

        
        Self {
            consumer,
            prev_frame: vec![0.0f32; 2048],
        }
    }
}

impl Widget for Graph {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas<OpenGl>) {
        
        // Skip window
        if entity == Entity::new(0, 0) {
            return;
        }

        // Skip invisible widgets
        if state.data.get_visibility(entity) == Visibility::Invisible {
            return;
        }

        if state.data.get_opacity(entity) == 0.0 {
            return;
        }

        let bounds = state.data.get_bounds(entity);

        let posx = bounds.x;
        let posy = bounds.y;
        let width = bounds.w;
        let height = bounds.h;

        //println!("entity: {} posx: {} posy: {} width: {} height: {}", entity, posx, posy, width, height);

        // Skip widgets with no width or no height
        // if width == 0.0 || height == 0.0 {
        //     return;
        // }

        let background_color = state
            .style
            .background_color
            .get(entity)
            .cloned()
            .unwrap_or_default();

        let font_color = state
            .style
            .font_color
            .get(entity)
            .cloned()
            .unwrap_or(tuix::Color::rgb(255, 255, 255));

        let border_color = state
            .style
            .border_color
            .get(entity)
            .cloned()
            .unwrap_or_default();

        let parent = state
            .hierarchy
            .get_parent(entity)
            .expect("Failed to find parent somehow");

        let parent_width = state.data.get_width(parent);

        let border_radius_top_left = match state.style.border_radius_top_left.get(entity).cloned().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => parent_width * val,
            _ => 0.0,
        };

        let border_radius_top_right = match state.style.border_radius_top_right.get(entity).cloned().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => parent_width * val,
            _ => 0.0,
        };

        let border_radius_bottom_left = match state.style.border_radius_bottom_left.get(entity).cloned().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => parent_width * val,
            _ => 0.0,
        };

        let border_radius_bottom_right = match state.style.border_radius_bottom_right.get(entity).cloned().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => parent_width * val,
            _ => 0.0,
        };

        let opacity = state.data.get_opacity(entity);

        let mut background_color: femtovg::Color = background_color.into();
        background_color.set_alphaf(background_color.a * opacity);

        let mut border_color: femtovg::Color = border_color.into();
        border_color.set_alphaf(border_color.a * opacity);


        let border_width = match state.style.border_width.get(entity).cloned().unwrap_or_default() {
            Units::Pixels(val) => val,
            Units::Percentage(val) => parent_width * val,
            _ => 0.0,
        };

        //println!("Border Width: {}", border_width);

        
        

        
        
        // Apply transformations
        let rotate = state.style.rotate.get(entity).unwrap_or(&0.0);
        let scaley = state.style.scaley.get(entity).cloned().unwrap_or_default();

        canvas.save();
        // canvas.translate(posx + width / 2.0, posy + height / 2.0);
        // canvas.rotate(rotate.to_radians());
        // canvas.translate(-(posx + width / 2.0), -(posy + height / 2.0));

        let pt = canvas.transform().inversed().transform_point(posx + width / 2.0, posy + height / 2.0);
        //canvas.translate(posx + width / 2.0, posy + width / 2.0);
        canvas.translate(pt.0, pt.1);
        canvas.scale(1.0, scaley.0);
        canvas.translate(-pt.0, -pt.1);


        // Apply Scissor
        let clip_bounds = state.data.get_clip_region(entity);

        canvas.scissor(clip_bounds.x, clip_bounds.y, clip_bounds.w, clip_bounds.h);

        // Draw rounded rect
        let mut path = Path::new();
        path.rounded_rect_varying(
            posx + (border_width / 2.0),
            posy + (border_width / 2.0),
            width - border_width,
            height - border_width,
            border_radius_top_left,
            border_radius_top_right,
            border_radius_bottom_right,
            border_radius_bottom_left,
        );
        let mut paint = Paint::color(background_color);
        canvas.fill_path(&mut path, paint);

        // Draw border
        let mut paint = Paint::color(border_color);
        paint.set_line_width(border_width);
        //paint.set_anti_alias(false);
        canvas.stroke_path(&mut path, paint);
        //println!("posx: {}", posx);

        // Draw Vertical Lines
        // Convert value to pixel position
        // 30 - 

        let min = 1.477121;
        let max = 4.3013;
        let range = max - min;


        for f in &frequencies {
            let t = (f - min) * (width - 80.0) / range;
            let mut path = Path::new();
            path.move_to(posx + 40.5 + t.ceil(), posy);
            path.line_to(posx + 40.5 + t.ceil(), posy + height);
            let mut paint = Paint::color(femtovg::Color::rgb(80, 80, 80));
            paint.set_line_width(1.0);
            canvas.stroke_path(&mut path, paint);
        }

        for g in 0..5 {
            let t = g as f32 * (height - 80.0) / 4.0;
            let mut path = Path::new();
            path.move_to(posx, posy + 40.5 + t.ceil());
            path.line_to(posx + width, posy + 40.5 + t.ceil());
            let mut paint = Paint::color(femtovg::Color::rgb(80, 80, 80));
            paint.set_line_width(1.0);
            canvas.stroke_path(&mut path, paint);

        }

        // 30 Hz Label
        //let t = (width - 40.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0, posy + height - 27.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0, posy + height - 20.0, "30 Hz", label_paint);

        // 100 Hz Label
        let t = (2.0 - min) * (width - 80.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + height - 27.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + height - 20.0, "100 Hz", label_paint);

        // 1 KHz Label
        let t = (3.0 - min) * (width - 80.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + height - 27.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + height - 20.0, "1 kHz", label_paint);

        // 10 KHz Label
        let t = (4.0 - min) * (width - 80.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + height - 27.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + height - 20.0, "10 kHz", label_paint);

        // 20 KHz Label
        let t = width - 80.0;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + height - 27.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + height - 20.0, "20 kHz", label_paint);

        // -12 dB Label
        let t = 0.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 47.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 40.0, "-12 dB", label_paint);

        // -6 dB Label
        let t = 1.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 47.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 40.0 - t, "-6 dB", label_paint);

        // 0 dB Label
        let t = 2.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 47.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 40.0 - t, "0 dB", label_paint);

        // 6 dB Label
        let t = 3.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 47.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 40.0 - t, "6 dB", label_paint);

        // 12 dB Label
        let t = 4.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 47.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(32, 32, 32));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(80,80,80));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 40.0 - t, "12 dB", label_paint);


        // Draw Spectrum

        let mut path = Path::new();
        path.move_to(posx + 40.5, posy + height);

        let mut consumer = self.consumer.borrow_mut();

        consumer.update();

        let output = consumer.output_buffer();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(4096);

        let mut buffer = vec![Complex{re: 0.0f32, im: 0.0f32}; 4096];

        for (elem, sample) in buffer.iter_mut().zip(output.iter()) {
            *elem = Complex{re: *sample, im: 0.0f32};
        }

        fft.process(&mut buffer);

        let scale = 64.0; //sqrt(4096)
        let real_buffer = buffer.into_iter().map(|sample| sample.norm()).collect::<Vec<f32>>();
        //let maximum = real_buffer.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(&32.0);

        let mut alpha = (-2.0f32).exp();

        for i in 0..real_buffer.len() / 2 {
            let mut f = (i as f32) * (22500.0 / 2048.0);
            f = f.log10();
            let t = (f - min) * (width - 80.0) / range;
            let sample = real_buffer[i] * alpha + self.prev_frame[i] * (1.0 - alpha);
            let db_val = 1.0 + (10.0 * (sample/scale).log10()).max(-100.0) / 100.0;
            path.line_to(posx + 40.5 + t as f32, posy + height + 30.0 - (db_val * height));
            self.prev_frame[i] = sample;
        }

        let mut paint = Paint::color(femtovg::Color::rgb(200, 50, 50));
        paint.set_line_join(femtovg::LineJoin::Bevel);


        canvas.stroke_path(&mut path, paint);

    
    }

}

// Control Point
pub struct ControlPoint {
    moving: bool,
    pos_down_left: f32,
    pos_down_top: f32,
}

impl ControlPoint {
    pub fn new() -> Self {
        Self {
            moving: false,
            pos_down_left: 0.0,
            pos_down_top: 0.0,
        }
    }
}

impl Widget for ControlPoint {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_width(state, Pixels(20.0))
            .set_height(state, Pixels(20.0))
            .set_border_radius(state, Pixels(10.0))
            .set_child_space(state, Stretch(1.0))
            .set_text(state, "1")
            .set_background_color(state, Color::rgb(100, 100, 100))
            .set_position_type(state, PositionType::SelfDirected)
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseDown(button) => {
                    if event.target == entity {
                        self.moving = true;
                        state.capture(entity);
                        self.pos_down_left = state.mouse.left.pos_down.0 - state.data.get_posx(entity);
                        self.pos_down_top = state.mouse.left.pos_down.1 - state.data.get_posy(entity);
                    }
                }

                WindowEvent::MouseUp(button) => {
                    if event.target == entity {
                        self.moving = false;
                        state.release(entity);
                    }
                }

                WindowEvent::MouseMove(x, y) => {
                    if event.target == entity {
                        if self.moving {
                            let parent = state.hierarchy.get_parent(entity).unwrap();
                            let parent_left = state.data.get_posx(parent);
                            let parent_top = state.data.get_posy(parent);
                            entity.set_left(state, Pixels(*x - parent_left - self.pos_down_left)).set_top(state, Pixels(*y - parent_top - self.pos_down_top));
                        
                        
                            state.insert_event(
                                Event::new(WindowEvent::Redraw).target(Entity::root()),
                            );
                        }
                    }
                }

                _=> {}
            }
        }
    }


}