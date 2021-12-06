pub mod lpsd;
pub use lpsd::*;

pub fn index_to_freq(i: f32, min: f32, max: f32, length: f32) -> f32 {
    return 10.0f32.powf(min + (i * (max - min) / length));
}

pub fn index_to_amp(i: f32, min: f32, max: f32, length: f32) -> f32 {
    return min + (i * (max - min) / length);
}

pub fn freq_to_index(freq: f32, min: f32, max: f32, length: f32) -> f32 {
    return ((freq.log10() - min) * length / (max - min)).round();
}

pub fn amp_to_index(amp: f32, min: f32, max: f32, length: f32) -> f32 {
    return ((amp - min) * length / (max - min)).round();
}
