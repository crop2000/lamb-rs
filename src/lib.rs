#![warn(clippy::nursery)]
use faust_ui::{UIGet, UISet};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;
mod buffer;
mod dsp_192k;
mod dsp_48k;
mod dsp_96k;
use crate::dsp_48k::{UIActive, UIPassive};
use buffer::*;
// use cyma::utils::{HistogramBuffer, MinimaBuffer, PeakBuffer, VisualizerBuffer};
use cyma::prelude::*;
use default_boxed::DefaultBoxed;

// this seems to be the number JUCE is using
// TODO: does this need to be set at runtime?
const MAX_SOUNDCARD_BUFFER_SIZE: usize = 32768;

mod editor;

//provide into for different parameters
impl From<UIActive> for dsp_96k::UIActive {
    fn from(value: UIActive) -> Self {
        Self::from_repr(value as usize).expect("infallible")
    }
}

impl From<UIActive> for dsp_192k::UIActive {
    fn from(value: UIActive) -> Self {
        Self::from_repr(value as usize).expect("infallible")
    }
}

impl From<dsp_48k::UIPassive> for dsp_96k::UIPassive {
    fn from(value: dsp_48k::UIPassive) -> Self {
        Self::from_repr(value as usize).expect("infallible")
    }
}

impl From<dsp_48k::UIPassive> for dsp_192k::UIPassive {
    fn from(value: dsp_48k::UIPassive) -> Self {
        Self::from_repr(value as usize).expect("infallible")
    }
}

// Define an enum to hold the DSP's for different sample rates
enum DspVariant {
    Dsp48k(Box<dsp_48k::LambRs>),
    Dsp96k(Box<dsp_96k::LambRs>),
    Dsp192k(Box<dsp_192k::LambRs>),
}
impl Default for DspVariant {
    fn default() -> Self {
        Self::Dsp48k(dsp_48k::LambRs::default_boxed())
    }
}

impl DspVariant {
    fn init(&mut self, sample_rate: i32) {
        match sample_rate {
            0..=48000 => *self = Self::Dsp48k(dsp_48k::LambRs::default_boxed()),
            48001..=96000 => *self = Self::Dsp96k(dsp_96k::LambRs::default_boxed()),
            _ => *self = Self::Dsp192k(dsp_192k::LambRs::default_boxed()),
        }
        match self {
            Self::Dsp48k(ref mut dsp) => dsp.init(sample_rate),
            Self::Dsp96k(ref mut dsp) => dsp.init(sample_rate),
            Self::Dsp192k(ref mut dsp) => dsp.init(sample_rate),
        }
    }

    fn get_param(&mut self, param: dsp_48k::UIPassive) -> f64 {
        match self {
            Self::Dsp48k(dsp) => param.get_value(dsp),
            Self::Dsp96k(dsp) => {
                std::convert::Into::<dsp_96k::UIPassive>::into(param).get_value(dsp)
            }
            Self::Dsp192k(dsp) => {
                std::convert::Into::<dsp_192k::UIPassive>::into(param).get_value(dsp)
            }
        }
    }

    fn set_param(&mut self, param: UIActive, value: f64) {
        match self {
            Self::Dsp48k(dsp) => param.set(dsp, value),
            Self::Dsp96k(dsp) => {
                std::convert::Into::<dsp_96k::UIActive>::into(param).set(dsp, value)
            }
            Self::Dsp192k(dsp) => {
                std::convert::Into::<dsp_192k::UIActive>::into(param).set(dsp, value)
            }
        }
    }

    fn compute(&mut self, count: i32, buffers: &mut [&mut [f64]]) {
        match self {
            Self::Dsp48k(ref mut dsp) => dsp.compute(count.try_into().unwrap(), buffers),
            Self::Dsp96k(ref mut dsp) => dsp.compute(count.try_into().unwrap(), buffers),
            Self::Dsp192k(ref mut dsp) => dsp.compute(count.try_into().unwrap(), buffers),
        }
    }
}

pub struct Lamb {
    params: Arc<LambParams>,
    // dsp_holder: DspHolder,
    dsp_variant: DspVariant,

    accum_buffer: TempBuffer,
    temp_output_buffer_gr_l: [f64; MAX_SOUNDCARD_BUFFER_SIZE],
    temp_output_buffer_gr_r: [f64; MAX_SOUNDCARD_BUFFER_SIZE],

    /// sample rate
    sample_rate: f32,

    // These buffers will hold the sample data for the visualizers.
    bus_l: Arc<MonoBus>,
    bus_r: Arc<MonoBus>,
    gr_bus_l: Arc<MonoBus>,
    gr_bus_r: Arc<MonoBus>,
    histogram_bus: Arc<MonoBus>,
}
impl Default for Lamb {
    fn default() -> Self {
        Self {
            params: Arc::new(LambParams::new()),

            dsp_variant: DspVariant::default(),
            accum_buffer: TempBuffer::default(),

            temp_output_buffer_gr_l: [0.0; MAX_SOUNDCARD_BUFFER_SIZE],
            temp_output_buffer_gr_r: [0.0; MAX_SOUNDCARD_BUFFER_SIZE],
            sample_rate: 48000.0,
            bus_l: Default::default(),
            bus_r: Default::default(),
            gr_bus_l: Default::default(),
            gr_bus_r: Default::default(),
            histogram_bus: Default::default(),
        }
    }
}

include!("params.rs");

impl Plugin for Lamb {
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();
    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];
    const EMAIL: &'static str = "bart@magnetophon.nl";
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const NAME: &'static str = "lamb";
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const VENDOR: &'static str = "magnetophon";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.accum_buffer.resize(2, MAX_SOUNDCARD_BUFFER_SIZE);
        self.sample_rate = buffer_config.sample_rate;
        // TODO: make sample_rate a local variable to speed this up?
        self.bus_l.set_sample_rate(buffer_config.sample_rate);
        self.bus_r.set_sample_rate(buffer_config.sample_rate);
        self.gr_bus_l.set_sample_rate(buffer_config.sample_rate);
        self.gr_bus_r.set_sample_rate(buffer_config.sample_rate);
        self.histogram_bus
            .set_sample_rate(buffer_config.sample_rate);

        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.dsp_variant.init(buffer_config.sample_rate as i32);

        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.bus_l.clone(),
            self.bus_r.clone(),
            self.gr_bus_l.clone(),
            self.gr_bus_r.clone(),
            self.histogram_bus.clone(),
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let count = buffer.samples() as i32;
        self.accum_buffer.read_from_buffer(buffer);

        let bypass: f64 = match self.params.bypass.value() {
            true => 1.0,
            false => 0.0,
        };
        self.dsp_variant.set_param(UIActive::Bypass, bypass);

        let latency_mode: f64 = match self.params.latency_mode.value() {
            LatencyMode::Minimal => 0.0,
            LatencyMode::Fixed => 1.0,
        };
        self.dsp_variant
            .set_param(UIActive::FixedLatency, latency_mode);
        self.dsp_variant
            .set_param(UIActive::InputGain, self.params.input_gain.value() as f64);
        self.dsp_variant
            .set_param(UIActive::Strength, self.params.strength.value() as f64);
        self.dsp_variant
            .set_param(UIActive::Thresh, self.params.thresh.value() as f64);
        self.dsp_variant
            .set_param(UIActive::Attack, self.params.attack.value() as f64);
        self.dsp_variant.set_param(
            UIActive::AttackShape,
            self.params.attack_shape.value() as f64,
        );
        self.dsp_variant
            .set_param(UIActive::Release, self.params.release.value() as f64);
        self.dsp_variant.set_param(
            UIActive::ReleaseShape,
            self.params.release_shape.value() as f64,
        );
        self.dsp_variant.set_param(
            UIActive::ReleaseHold,
            self.params.release_hold.value() as f64,
        );
        self.dsp_variant
            .set_param(UIActive::Knee, self.params.knee.value() as f64);
        self.dsp_variant
            .set_param(UIActive::Link, self.params.link.value() as f64);
        self.dsp_variant.set_param(
            UIActive::AdaptiveRelease,
            self.params.adaptive_release.value() as f64,
        );
        self.dsp_variant
            .set_param(UIActive::Lookahead, self.params.lookahead.value() as f64);
        self.dsp_variant
            .set_param(UIActive::OutputGain, self.params.output_gain.value() as f64);

        let [io_buffer_l, io_buffer_r] = &mut self.accum_buffer.slice2d();
        let buffers = &mut [
            io_buffer_l,
            io_buffer_r,
            self.temp_output_buffer_gr_l.as_mut_slice(),
            self.temp_output_buffer_gr_r.as_mut_slice(),
        ];

        self.dsp_variant.compute(count, buffers);
        let latency_samples = self.dsp_variant.get_param(UIPassive::Latency) as u32;
        context.set_latency_samples(latency_samples);

        if self.params.editor_state.is_open() {
            if self.params.in_out.value() {
                for i in 0..count as usize {
                    self.bus_l.send(io_buffer_l[i] as f32);
                    self.bus_r.send(io_buffer_r[i] as f32);
                }
            } else {
                // TODO: document why this is done by reversing the effect of the dsp
                // was it so that the latency is accounted for?
                let gain_db =
                    0.0 - (self.params.input_gain.value() + self.params.output_gain.value());
                let gain = if self.params.bypass.value() {
                    1.0
                } else {
                    10f32.powf(gain_db / 20.0)
                };
                for i in 0..count as usize {
                    self.bus_l
                        .send((io_buffer_l[i] / self.temp_output_buffer_gr_l[i]) as f32 * gain);
                    self.bus_r
                        .send((io_buffer_r[i] / self.temp_output_buffer_gr_r[i]) as f32 * gain);
                }
            };
            for i in 0..count as usize {
                self.gr_bus_l.send(self.temp_output_buffer_gr_l[i] as f32);
                self.gr_bus_r.send(self.temp_output_buffer_gr_r[i] as f32);
            }
            // TODO: make this react to in_out?
            self.histogram_bus.send_buffer_summing(buffer);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Lamb {
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A lookahead compressor/limiter that's soft as a lamb");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Compressor,
        ClapFeature::Limiter,
        ClapFeature::Mastering,
    ];
    const CLAP_ID: &'static str = "magnetophon.nl lamb";
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
}

impl Vst3Plugin for Lamb {
    const VST3_CLASS_ID: [u8; 16] = *b"magnetophon lamb";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Dynamics,
        Vst3SubCategory::Mastering,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(Lamb);
nih_export_vst3!(Lamb);
