

use ringbuf::{Consumer, Producer};
use triple_buffer::{Input, Output, TripleBuffer};
use atomic_refcell::AtomicRefCell;
use tuix::*;
use femtovg::{renderer::OpenGl, Path, Paint, Align, Baseline, Canvas};
use rustfft::{Fft, FftPlanner, num_complex::Complex, num_traits::real};

use std::{cell::UnsafeCell, sync::Arc};
use std::cmp::Ordering;

use crate::{eq_core::svf::{Type, SVFCoefficients, ZSample}, eq_params::EQEffectParameters};
use crate::eq_core::{eq::{FilterKind, FilterbandStereo}};
use super::EQEvent;

use super::super::util::lpsd::lpsd;
use crate::util::*;

use crate::atomic_f64::AtomicF64;

const frequencies: [f32; 28] = [1.301030, 1.477121, 1.60206, 1.69897, 1.778151, 1.845098, 1.90309, 1.954243, 2.0, 2.30103, 2.477121, 2.60206, 2.69897, 2.778151, 2.845098, 2.90309, 2.954243, 3.0, 3.30103, 3.477121, 3.60206, 3.69897, 3.778151, 3.845098, 3.90309, 3.954243, 4.0, 4.30103];

#[derive(Debug,Copy,Clone)]
struct Params {
    kind: FilterKind,
    gain: f32,
    freq: f32,
    q: f32,
}

pub struct Graph {
    filter_bands: [FilterbandStereo; 4],

    bode_plots: [[f32; 720]; 4],

    params: Arc<EQEffectParameters>,
    sample_rate: Arc<AtomicF64>,  

    //params: [Params; 4],
}

impl Graph {
    pub fn new(params: Arc<EQEffectParameters>, sample_rate: Arc<AtomicF64>) -> Self {
        Self {

            filter_bands: [FilterbandStereo::new(44100.0); 4],

            bode_plots: [[0.0; 720]; 4],

            params: params.clone(),
            
            sample_rate: sample_rate.clone(),

        }
    }
}

impl Widget for Graph {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(eq_event) = event.message.downcast::<EQEvent>() {
            match eq_event {
                EQEvent::MovePoint(index,x,y) => {

                    let xx = *x - 40.0;
                    let yy = *y - 40.0;

                    let freq = index_to_freq(xx, 1.301030, 4.3013, 720.0);
                    let gain = index_to_amp(yy, 12.0, -12.0, 370.0);

                    //println!("{} {}", freq , amp);
                    
                    //self.filters[*index] = SVFCoefficients::<f64>::from_params(Type::PeakingEQ(amp as f64), 44100.0, freq as f64, 1.0).unwrap();
                    //self.params[*index] = Params {kind: self.params[*index].kind, gain: amp, freq: freq, q: 1.0};
                    //self.params.bands[*index].gain.set(gain as f64);
                    //self.params.bands[*index].freq.set(freq as f64);
                }

                // EQEvent::SetFreq(index, freq) => {
                //     let params = self.params[*index];
                //     self.filters[*index] = SVFCoefficients::<f64>::from_params(Type::PeakingEQ(params.gain as f64), 44100.0, *freq as f64, 1.0).unwrap();
                //     self.params[*index].freq = *freq;
                // }

                // EQEvent::SetGain(index, gain) => {
                //     let params = self.params[*index];
                //     self.filters[*index] = SVFCoefficients::<f64>::from_params(Type::PeakingEQ(*gain as f64), 44100.0, params.freq as f64, 1.0).unwrap();
                //     self.params[*index].gain = *gain;
                // }

                // EQEvent::SetKind(index, kind) => {
                //     let params = self.params[*index];
                //     self.filters[*index] = SVFCoefficients::<f64>::from_params(kind, 44100.0, params.freq as f64, 1.0).unwrap();
                //     self.params[*index].kind = *kind;
                // }

                // EQEvent::UpdateFilters(filters) => {
                //     for (index, filter) in filters.iter().enumerate() {
                //         let old = self.filter_bands[index];
                //         if old != *filter {
                //             for i in (0..720) {
                //                 // TODO: Cache this
                //                 let min = 1.477121;
                //                 let max = 4.3013;
                //                 let freq = index_to_freq(i as f32, min, max, 720.0);
                //                 let z = ZSample::new(freq as f64, 44100.0);
                //                 let amp = filter.get_bode_sample(z).norm() as f32;
                //                 let amp_db = 20.0 * amp.log10().max(-12.0) / 12.0;
                //                 self.bode_plots[index][i] = amp_db;
                //             }
                //         }
                //         self.filter_bands[index] = *filter;
                //     }
                // }

                _=> {}
            }
        }
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

        let min = 1.301030;
        let max = 4.30103;
        let range = max - min;


        for f in &frequencies {
            let t = (f - min) * (width - 80.0) / range;
            let mut path = Path::new();
            path.move_to(posx + 40.5 + t.ceil(), posy);
            path.line_to(posx + 40.5 + t.ceil(), posy + height);
            // let mut paint = Paint::color(femtovg::Color::rgb(95, 87, 87));
            let mut paint = Paint::linear_gradient_stops(
                0.0, 
                0.0, 
                0.0, 
                height,
                &[
                        (0.0, femtovg::Color::rgb(23,18,21)),
                        (0.1, femtovg::Color::rgb(58, 53, 54)),
                        (0.9, femtovg::Color::rgb(58, 53, 54)),
                        (1.0, femtovg::Color::rgb(23,18,21))
                    ]);
            paint.set_line_width(1.0);
            canvas.stroke_path(&mut path, paint);
        }

        for g in 0..5 {
            let t = g as f32 * (height - 80.0) / 4.0;
            let mut path = Path::new();
            path.move_to(posx, posy + 40.5 + t.ceil());
            path.line_to(posx + width, posy + 40.5 + t.ceil());
            //let mut paint = Paint::color(femtovg::Color::rgb(95, 87, 87));
            let mut paint = Paint::linear_gradient_stops(
                0.0, 
                0.0, 
                width,
                0.0, 
                &[
                        (0.0, femtovg::Color::rgb(23,18,21)),
                        (0.1, femtovg::Color::rgb(58, 53, 54)),
                        (0.9, femtovg::Color::rgb(58, 53, 54)),
                        (1.0, femtovg::Color::rgb(23,18,21))
                    ]);
            paint.set_line_width(1.0);
            canvas.stroke_path(&mut path, paint);

        }

        // 30 Hz Label
        //let t = (width - 40.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0, posy + 10.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0, posy + 16.0, "30 Hz", label_paint);

        // 100 Hz Label
        let t = (2.0 - min) * (width - 80.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + 10.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + 16.0, "100 Hz", label_paint);

        // 1 KHz Label
        let t = (3.0 - min) * (width - 80.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + 10.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + 16.0, "1 kHz", label_paint);

        // 10 KHz Label
        let t = (4.0 - min) * (width - 80.0) / range;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + 10.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23, 18, 21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + 16.0, "10 kHz", label_paint);

        // 20 KHz Label
        let t = width - 80.0;
        let mut path = Path::new();
        path.rect(posx + 30.0 + t, posy + 10.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 49.0 + t, posy + 16.0, "20 kHz", label_paint);

        // -12 dB Label
        let t = 0.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 55.0, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 48.0, "-12 dB", label_paint);

        // -6 dB Label
        let t = 1.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 55.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 48.0 - t, "-6 dB", label_paint);

        // 0 dB Label
        let t = 2.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 55.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 48.0 - t, "0 dB", label_paint);

        // 6 dB Label
        let t = 3.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 55.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 48.0 - t, "6 dB", label_paint);

        // 12 dB Label
        let t = 4.0 * (height - 80.0) / 4.0;
        let mut path = Path::new();
        path.rect(posx, posy + height - 55.0 - t, 40.0, 14.0);
        let mut paint = Paint::color(femtovg::Color::rgb(23,18,21));
        canvas.fill_path(&mut path, paint);
        let mut label_paint = Paint::color(femtovg::Color::rgb(179, 172, 172));
        label_paint.set_text_align(femtovg::Align::Center);
        label_paint.set_text_baseline(Baseline::Middle);
        label_paint.set_font_size(12.0);
        canvas.fill_text(posx + 20.0, posy + height - 48.0 - t, "12 dB", label_paint);



        


        // let mut consumer = self.filterband_consumer.borrow_mut();
        // consumer.update();
        // let output = consumer.output_buffer();

        // // Cache the bode samples
        // for (index, filter) in output.iter().enumerate() {
        //     let old = self.filter_bands[index];
        //     if old != *filter {
        //         for i in (0..720) {
        //             // TODO: Cache this
        //             let freq = index_to_freq(i as f32, min, max, 720.0);
        //             let z = ZSample::new(freq as f64, 44100.0);
        //             let amp = filter.get_bode_sample(z).norm() as f32;
        //             let amp_db = 20.0 * amp.log10().max(-12.0) / 12.0;
        //             self.bode_plots[index][i] = amp_db;
        //         }
        //     }
        //     self.filter_bands[index] = *filter;
        // }

        
        
        // Draw Spectrum (TODO)

        // let mut path = Path::new();
        // path.move_to(posx + 40.5, posy + height);



        // let mut consumer = self.consumer.borrow_mut();

        // consumer.update();

        // let output = consumer.output_buffer();


        // let real_buffer = lpsd(&output, 30.0, 20000.0, 720, 100, 2, 44100.0, 0.5);

        // // let mut planner = FftPlanner::new();
        // // let fft = planner.plan_fft_forward(512);

        // // let mut buffer = vec![Complex{re: 0.0f32, im: 0.0f32}; 512];

        // // for (elem, sample) in buffer.iter_mut().zip(output.iter()) {
        // //     *elem = Complex{re: *sample, im: 0.0f32};
        // // }

        // // fft.process(&mut buffer);

        // let scale = 64.0; //sqrt(512)
        // //let real_buffer = buffer.into_iter().map(|sample| sample.norm()).collect::<Vec<f32>>();
        // // let maximum = real_buffer.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(&32.0);

        // let mut alpha = (-2.0f32).exp();

        // // for i in 0..real_buffer.len() / 2 {
        // //     let mut f = (i as f32) * (22500.0 / 2048.0);
        // //     f = f.log10();
        // //     let t = (f - min) * (width - 80.0) / range;
        // //     let sample = real_buffer[i] * alpha + self.prev_frame[i] * (1.0 - alpha);
        // //     let db_val = 1.0 + (10.0 * (sample/scale).log10()).max(-100.0) / 100.0;
        // //     path.line_to(posx + 40.5 + t as f32, posy + height + 30.0 - (db_val * height));
        // //     self.prev_frame[i] = sample;
        // // }
        
        // for i in 0..720 {

        //     //let log_freq = log_index(i as f32, min, max);
        //     //let index = freq_to_index(log_freq, 44100.0, 512);

        //     // let low = index.floor();
        //     // let high = index.ceil();
        //     // let low_val = real_buffer[low as usize];
        //     // let high_val = real_buffer[high as usize];
        //     // let w = (index - low) / (high - low);
        //     // let v = low_val + (high_val - low_val) * w;
        //     let sample = real_buffer[i] * alpha + self.prev_frame[i] * (1.0 - alpha);
        //     // let sample = real_buffer[i];
        //     let db_val = 1.0 + (10.0 * (sample/512.0).log10()).max(-100.0) / 100.0;
        //     if i == 0 {
        //         path.move_to(posx + 40.0 + i as f32, posy + height + 40.0 - (db_val * height));
        //     } else {
        //         path.line_to(posx + 40.0 + i as f32, posy + height + 40.0 - (db_val * height));
        //     }
        //     self.prev_frame[i] = sample;
        // }

        // let mut paint = Paint::color(femtovg::Color::rgb(234, 189, 106));
        // paint.set_line_join(femtovg::LineJoin::Bevel);
        // canvas.stroke_path(&mut path, paint);


        // Draw bode plot
        let height_span = height - 80.0;

        let sample_rate = self.sample_rate.get();

        let mut y_values = vec![0.0f32; 720];


        
        for (index, band) in self.params.bands.iter().enumerate() {

            

            let mut new_band = FilterbandStereo::new(sample_rate);
            new_band.update(
                band.get_kind(),
                band.freq.get(),
                band.gain.get(),
                band.bw.get(),
                band.get_slope(),
                sample_rate,
            );

            let mut fill_path = Path::new();
            let mut stroke_path = Path::new();
            fill_path.move_to(posx + 40.5, posy + 40.0 + height_span / 2.0);
            for i in (0..720) {
                let freq = index_to_freq(i as f32, min, max, 720.0);
                let z = ZSample::new(freq as f64, 44100.0);
                let amp = new_band.get_bode_sample(z).norm() as f32;
                let amp_db = 20.0 * amp.log10().max(-12.0) / 12.0;

                y_values[i] += amp_db as f32;
                //let amp = self.filter_bands[index].get_amplitude(freq as f64) as f32;
                //let amp_db = 20.0 * amp.log10().max(-12.0) / 12.0;
                //let amp_db = self.bode_plots[index][i];
                fill_path.line_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
                if i == 0 {
                    stroke_path.move_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
                } else {
                    stroke_path.line_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
                }
            }
            fill_path.line_to(posx + 40.5 + 720.0,posy + 40.0 + height_span / 2.0);
            fill_path.line_to(posx + 40.5, posy + 40.0 + height_span / 2.0);
            path.close();

            
            let paint = if index == 0 {
                Paint::color(femtovg::Color::rgba(64, 67, 246, 126))
            } else if index == 1 {
                Paint::color(femtovg::Color::rgba(180, 67, 246, 126))
            } else if index == 2{
                Paint::color(femtovg::Color::rgba(246, 67, 246, 126))
            } else {
                Paint::color(femtovg::Color::rgba(140, 246, 67, 126))
            };
            canvas.fill_path(&mut fill_path, paint);

            let mut paint = if index == 0 {
                Paint::color(femtovg::Color::rgb(64, 67, 246))
            } else if index == 1 {
                Paint::color(femtovg::Color::rgb(180, 67, 246))
            } else if index == 2 {
                Paint::color(femtovg::Color::rgb(246, 67, 246))
            } else {
                Paint::color(femtovg::Color::rgb(140, 246, 67))
            };

            paint.set_line_join(femtovg::LineJoin::Bevel);
            paint.set_line_width(1.0);
            canvas.stroke_path(&mut stroke_path, paint);
        }




        // let mut path = Path::new();
        // (0..720).map(|index| index_to_freq(index as f32, min, max, 720.0)).enumerate().map(|(i, f)| {
        //     let amp_db = (20.0 * self.filter_bands.iter().fold(0.0, |hf, h| hf * h.get_amplitude(f as f64))
        //     .log10()
        //     .max(-12.0) / 12.0) as f32;

        //     if i == 0 {
        //         path.move_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
        //     } else {
        //         path.line_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
        //     }

        // });
        
        // let mut path = Path::new();
        // for i in 0..720 {
        //     let freq = index_to_freq(i as f32, min, max, 720.0);
        //     let z = ZSample::new(freq as f64, 44100.0); //TODO get actual sample rate
        //     let amp0 = self.filters[0].get_bode_sample(z).norm() as f32;
        //     let amp1 = self.filters[1].get_bode_sample(z).norm() as f32;
        //     let amp2 = self.filters[2].get_bode_sample(z).norm() as f32;

        //     let amp = amp0 * amp1 * amp2;
            

        //     let amp = (amp0 * amp1 * amp2 * amp3);
            
        //     let amp_db = 20.0 * amp.log10().max(-12.0) / 12.0;
        //     if i == 0 {
        //         path.move_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
        //     } else {
        //         path.line_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
        //     }
            
        // }
        

        let mut path = Path::new();
        for i in 0..720 {

            let amp_db = y_values[i];
            
            if i == 0 {
                path.move_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
            } else {
                path.line_to(posx + 40.5 + i as f32, posy + 40.0 + height_span/2.0 - amp_db * height_span/2.0);
            }
            
        }
        

        let mut paint = Paint::color(femtovg::Color::rgb(246, 67, 64));
        paint.set_line_join(femtovg::LineJoin::Bevel);
        paint.set_line_width(2.0);
        canvas.stroke_path(&mut path, paint);
    }

}

// Control Point
pub struct ControlPoint {
    moving: bool,
    pos_down_left: f32,
    pos_down_top: f32,
    text: String,
    pub px: f32,
    pub py: f32,
    minx: f32,
    maxx: f32,
    miny: f32,
    maxy: f32,
    on_press: Option<Box<dyn Fn(&Self, &mut State, Entity)>>,
    on_release: Option<Box<dyn Fn(&Self, &mut State, Entity)>>,
    on_move: Option<Box<dyn Fn(&Self, &mut State, Entity)>>,
}

impl ControlPoint {
    pub fn new(text: &str) -> Self {
        Self {
            moving: false,
            pos_down_left: 0.0,
            pos_down_top: 0.0,
            px: 0.0,
            py: 0.0,
            minx: 40.0,
            maxx: 760.0,
            miny: 40.0,
            maxy: 410.0,
            on_press: None,
            on_release: None,
            on_move: None,
            text: text.to_owned(),
        }
    }

    pub fn on_press<F>(mut self, message: F) -> Self
        where F: 'static + Fn(&Self, &mut State, Entity),
    {
        self.on_press = Some(Box::new(message));

        self
    }

    pub fn on_release<F>(mut self, message: F) -> Self
        where F: 'static + Fn(&Self, &mut State, Entity),
    {
        self.on_release = Some(Box::new(message));

        self
    }

    pub fn on_move<F>(mut self, message: F) -> Self
        where F: 'static + Fn(&Self, &mut State, Entity),
    {
        self.on_move = Some(Box::new(message));

        self
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
            .set_text(state, &self.text)
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

                        let parent = state.hierarchy.get_parent(entity).unwrap();
                        let parent_left = state.data.get_posx(parent);
                        let parent_top = state.data.get_posy(parent);

                        let width = state.data.get_width(entity);
                        let height = state.data.get_height(entity);

                        self.px = state.mouse.left.pos_down.0 - parent_left - self.pos_down_left + width / 2.0;
                        self.py = state.mouse.left.pos_down.1 - parent_top - self.pos_down_top + height / 2.0;

                        self.px = self.px.clamp(self.minx, self.maxx);
                        self.py = self.py.clamp(self.miny, self.maxy);

                        if let Some(on_press) = &self.on_press {
                            (on_press)(self, state, entity);
                        }
                    }
                }

                WindowEvent::MouseUp(button) => {
                    if event.target == entity {
                        self.moving = false;
                        state.release(entity);

                        if let Some(on_release) = &self.on_release {
                            (on_release)(self, state, entity);
                        }
                    }
                }

                WindowEvent::MouseMove(x, y) => {
                    if event.target == entity {
                        if self.moving {
                            let parent = state.hierarchy.get_parent(entity).unwrap();
                            let parent_left = state.data.get_posx(parent);
                            let parent_top = state.data.get_posy(parent);

                            let width = state.data.get_width(entity);
                            let height = state.data.get_height(entity);
                            
                        
                            self.px = *x - parent_left - self.pos_down_left + width / 2.0;
                            self.py = *y - parent_top - self.pos_down_top + height / 2.0;

                            self.px = self.px.clamp(self.minx, self.maxx);
                            self.py = self.py.clamp(self.miny, self.maxy);

                            entity
                                .set_left(state, Pixels(self.px - width / 2.0))
                                .set_top(state, Pixels(self.py - height / 2.0));

                            if let Some(on_move) = &self.on_move {
                                (on_move)(self, state, entity);
                            }
                        
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