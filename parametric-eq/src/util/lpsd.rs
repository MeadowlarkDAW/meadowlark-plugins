use num_complex::Complex;


pub fn hanning(N: usize) -> Vec<f32> {

    let mut w = vec![0.0; N];

    let n = N;
    if n%2 == 0 {
        let half = n/2;
        
        for i in 0..half {
            w[i] = 0.5 * (1.0 - (2.0*std::f32::consts::PI*(i as f32+1.0) / (n as f32 + 1.0)).cos());
        }

        let mut idx = half.overflowing_sub(1).0;
        for i in half..n {
            w[i] = w[idx];
            idx = idx.overflowing_sub(1).0;
        }
    } else {
        let half = (n+1)/2;
        for i in 0..half {
            w[i] = 0.5 * (1.0 - (2.0*std::f32::consts::PI*(i as f32+1.0) / (n as f32 + 1.0)).cos());
        }

        let mut idx = half.overflowing_sub(2).0;
        for i in half..n {
            w[i] = w[idx];
            idx = idx.overflowing_sub(1).0;
        }
    }

    return w;
}

pub fn lpsd(x: &[f32], fmin: f32, fmax: f32, Jdes: u32, Kdes: u32, Kmin: u32, Fs: f32, xi: f32) -> Vec<f32> {
    // Inputs
    // x - time series data
    // fmin - lowest frequency to estimate
    // fmax - highest frequency to estimate
    // Jdes - desired number of Fourier frequencies
    // Kdes - desired number of averages
    // Kmin - minimum number of averages
    // Fs - sample rate
    // xi - fractional overalp between segments

    let N = x.len();
    let g = fmax.ln() - fmin.ln();

    let mut pxx = vec![0.0; Jdes as usize];

    for j in 0..Jdes as usize {

        let jj = j as f32;
        
        let f = fmin * (jj * g / (Jdes - 1) as f32).exp();
        
        
        let rp = fmin * (jj * g / (Jdes - 1) as f32).exp() * ((g  / (Jdes - 1) as f32).exp() - 1.0);
        
        
        let ravg = (Fs / N as f32) * (1.0 + (1.0 - xi) * (Kdes - 1) as f32);
        
        
        let rmin = (Fs / N as f32) * (1.0 + (1.0 - xi) * (Kmin - 1) as f32);
        

        let rpp = if rp >= ravg {
            rp
        } else if rmin <= rp && rp < ravg {
            (ravg * rp).sqrt()
        } else {
            rmin
        };

        
        let L = (Fs / rpp).round(); // Segment length
        
        let r = Fs / L;
        
        let m = f / r;


        // Calculate number of segments
        let D = ((1.0 - xi) * L).round();
        let K = ((N as f32 - L) / D + 1.0).floor() as usize;
        
        let mut window = hanning(L as usize);
        
        let mut mean_value = 0.0;

        for k in 0..K {
            let segment_start = k * D as usize;
            let segment_end = segment_start + L as usize;
            let segment = &x[segment_start..segment_end];
            
            
            let mean = segment.iter().sum::<f32>() / L;

            let mut segment = segment.to_owned();

            // Remove the mean of each segment
            segment.iter_mut().for_each(|sample| *sample =  *sample - mean);
            
            let data = segment
                .iter()
                .enumerate()
                .map(|(index, sample)| {
                    let sinusoid = (-2.0 * std::f32::consts::PI * Complex::i() * index as f32 * (m/L)).exp();
                    sample * sinusoid * window[index]
                }).sum::<Complex<f32>>().norm();
            
            
            let mag = data * data;
            
            mean_value += mag;
        }

        mean_value /= K as f32;

        pxx[j] = mean_value;
        
    }

    pxx

}