# Meadowlark Plugins Design Document

Meadowlark aims to be a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. Its goals are to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.

# Objective

Our main focus for Meadowlark's suite of internal plugins will be on a suite of essential mixing and mastering FX plugins. (Contribution on synths and other exotic effects are welcome, but they are not a priority right now).

We obviously don't have the resources to compete with the DSP quality of companies like Waves, iZotope, or Fabfilter. Our aim is have "good enough" quality to where a producer can create a "pretty good" mix using Meadowlark's internal plugins alone. Essentially I want beginning producers to not feel like they have to hunt down external plugins for each part of the music production process. In addition this will make it much easier to create music production tutorials using Meadowlark's internal plugins alone.

Because we have a small team at the moment, we will focus more on porting DSP from other existing open source plugins to Rust rather than doing all of the R&D from scratch ourselves. People can still do their own R&D if they wish (and there are cases where we have to because an open source plugin doesn't exist for that case), but there already exists some great DSP in the open source world (especially in synth [`Vital`]). I've noted other open source plugins we can port the DSP in the `Plugin Suite` section below.

One more note on DSP. We should always consider replacing any biquad filters with the [`SVF`] filter as it behaves much better when being automated and tends to be more stable with high Q factors.

# Special note on using Rust

While we definitely prefer writing in Rust, if you wish to contribute DSP and you aren't comfortable with Rust, you are free to develop the DSP in whatever programming language you are comfortable with (On the one condition that the DSP be completely self-contained and not depend on any external DSP libraries). We can then port your code to Rust once you are done.

# Non-Goals

Note that we should avoid creating a "reusable DSP library". I believe those to be more of a hassle than they are worth, and they also serve to deter DSP experimentation and optimizations when developing plugins. Each plugin should have its own standalone and optimized DSP. We are of course still allowed to copy-paste portions of DSP between plugins as we see fit.

Also note that while the architecture of [`dropseed`] techinally treats every node in the application as a "plugin", only audio FX and synth plugins will live in this repo. Nodes like the "timeline track plugin", "sample browser playback plugin", "metronome plugin", "mixer plugin", and the "send plugin" in Meadowlark will live in the Meadowlark repo itself.

# MVP Goals

For MVP, we will only focus only on creating a few very basic plugins as a proof of concept. These simple plugins will include:
* A noise generator (generates white, pink, and brown noise)
* A gain/pan utitlity plugin
* A sine test tone generator

# Plugin Framework

Plugins will be created using the [`nih-plug`] framework.

In order to have feature-rich inline UIs in Meadowlark's horizontal FX rack, we will use our own custom CLAP extension that lives in the [`meadowlark-clap-exts`] repo *(This extension has yet to be written).* This allows us to only need one plugin framework for both the "used inside Meadowlark" target and the "used outside of Meadowlark" target.

# Plugin Suite

Here I'll list the suite of plugins I want for Meadowlark.

Each plugin is marked with a priority, with 5 asteriks (`*****`) being the highest priority, and one asterisk (`*`) being the lowest priority.

Also note that while I do reference [`Vital`] a lot here as a source of DSP, from now on I will reference the fully-GPL-licensed fork called [`Vitalium`]. This will ensure thate whatever we are porting from this plugin is licensed under GPL.

# Audio FX Suite

## [ ] Gain/pan utility plugin
priority: `*****`

This will be a very basic plugin that just has a single gain knob and a single pan knob. DSP for this should be very straight-forward.

## [ ] Parametric EQ
priority: `*****`

The parametric EQ is the most important and essential plugin in any kind of music production workflow, so it's important that we get this one right.

### DSP

This plugin should have the essentials:
* A lowshelf/lowpass band
    * The lowpass band should include various slopes including 6dB/oct, 12dB/oct, 24dB/oct, 36dB/oct, and 48dB/oct.
* A high-shelf/highpass band
    * The highpass band should include various slopes including 6dB/oct, 12dB/oct, 24dB/oct, 36dB/oct, and 48dB/oct.
* A variable number of bell/notch filter bands
* A spectrometer that can show either pre eq, post eq, or be off
    * Note that I personally prefer to have the "FL-Studio" style of spectrometer of using color to represent frequency intensity instead of the traditional "line graph" approach. I feel the latter gives a false impression of how humans actually hear sound, and it can lead to bad habits of "mixing with your eyes". I really like FL's approach here because the "wash of color" more accurately represents how our brains actually perceive frequencies as a "wash of frequencies". 
    * ![FL EQ screenshot](images/design_doc/fl_eq.jpg)

In addition I would like this eq to have two toggles:
* A toggle that turns on "adpative Q" mode. When enabled, it automatically narrows the Q factor as the gain of a band increases. This has the effect of sounding more "musical" to some people.
* If we go with a DSP algorithm that introduces latency, then we should add a "high quality" toggle (should be enabled by default) that switches between using latency for better quality, or using zero latency at the cost of worse quality.

Two more notes on quality:
* Frequencies should not "cramp" near the high end. For more info check out [this article](https://www.pro-tools-expert.com/production-expert-1/what-is-eq-cramping-and-should-you-care).
* The filters must behave well when being automated quickly (EQs are commonly used as filter effects in electronic music production).

### UI/UX

My favorite workflow for an EQ is that of Bitwig's older EQ5 plugin (I hate the workflow of their newer EQ+ plugin). Obviously having its UI be inline has its advantages, but the main reason I prefer it is because it exposes the gain/frequency/q parameters for *all* of the bands at once, not just the currently selected band. I think we should mostly follow this design for our EQ.

![EQ5](images/design_doc/EQ5.png)

In addition we should add the ability to add more that 5 bands. We can have the inline UI automatically grow in size to acommadate the new bands. Because we are using the CLAP spec and our own UI library, this shouldn't be too big of an issue for us.

### Non-goals

This plugin will have no mid/side mode. This is because the user can easily construct a mid/side EQ by placing two EQs into the mid/side splitter plugin.

This plugin will have no "dynamic EQ" mode, since this would introduce a lot of complexity to our already cramped UI. The dynamic EQ will be its own plugin.

### Existing DSP References
* [`x42-fil4`](https://github.com/x42/fil4.lv2/tree/master) is shipped with the commercial Harrison Mixbus DAW, so it has great potential in the quality of its filters. It even states in its readme that it behaves well while being automated and it doesn't "cramp" near Nyquist, so already it checks a lot of boxes for us.
    * One thing I would change is to replace the biquad filters used for the high/low shelves with the [`SVF`] filter for better behavior when being automated.
* [`Vitalium`] has an [`equalizer module`](https://github.com/DISTRHO/DISTRHO-Ports/blob/master/ports-juce6/vitalium/source/synthesis/modules/equalizer_module.h) with 3 bands. I'm not certain on the quality of the filters used here, but they could be good.


## [ ] Single-band Compressor
priority: `*****`

### DSP

This will be a basic single-band compressor with the standard parameters:
* input gain
* attack
* release
* threshold
* ratio
* knee
* Peak/RMS mode (depending on the algorithm used)
* output gain

This plugin should also have an optional sidechain input with a basic lowpass and highpass filter applied to the sidechain signal.

In addition, this plugin should have a parameter that can switch between different compression algorithms (aka "colors"). This will allow us to easily add new and improved algorithms in the future without having to create an entirely new plugin for each addition *(looking at you FL)*.

### UI/UX

I have no particular preferences on the design or workflow of this plugin. We can go simple with just knobs and a basic gain reduction meter, or we could go fancy with a waveform view that graphs the gain reduction in realtime. The one stipulation I have is to have a dropdown to select between different compression algorithms (we can refer to them as "colors").

### Existing DSP References
* [`x42-darc`](https://github.com/x42/darc.lv2/tree/master) - This is another nice sounding compressor. This is probably a good one to have as the default "color".
* [`zamcompx2`](https://github.com/zamaudio/zamcomp) - I really like the sound of this compressor when applied to transient material like durms. We can call this "color" something like "ZAM" or "Punchy".
* [`Pressure4`](https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/Pressure4) - This is probably Airwindow's most popular plugin and for good reason. We can call this "color" something like "Pressure".

## [ ] Dynamics Processor
priority: `****`

This is essentially a compressor but with the added ability to expand in addition to compress. This plugin will be able to switch between single-band mode and multi-band mode (3 bands).

### DSP

The compressor module in [`Vitalium`] is already exactly what I'm looking for, so I think we should simply port this module to Rust. The only addition we need to make is to add the crossover frequencies as parameters.

### UI/UX

The workflow should work like that of Vitalium's compressor module / Ableton's Multiband Dynamics plugin. Each band has a dB meter for that band, along with rectangles that represent the range where expansion/compression is applied. The user can drag these rectangles to adjust the threshhold of the expansion/compression, as well as drag inside the rectangle to adjust the "ratio" of the expansion/compression.

In addition the UI should have a toggle to switch between single-band mode and multi-band mode, as well as knobs to adjust the crossover frequencies when in multi-band mode.

![Vitalium Compressor Module](images/design_doc/vitalium_comp_module.png)
![Ableton Multiband Dynamics Plugin](images/design_doc/ableton_multiband_dynamics.png)

---

*TODO: Rest of the plugins*

[`nih-plug`]: https://github.com/robbert-vdh/nih-plug
[`meadowlark-clap-exts`]: https://github.com/MeadowlarkDAW/meadowlark-clap-exts
[`dropseed`]: https://github.com/MeadowlarkDAW/dropseed
[`Vital`]: https://github.com/mtytel/vital
[`Vitalium`]: https://github.com/DISTRHO/DISTRHO-Ports/tree/master/ports-juce6/vitalium
[`SVF`]: https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs