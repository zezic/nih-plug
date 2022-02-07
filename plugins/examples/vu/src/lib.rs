// vu: vu meter using nih-plugins
// Author: Sergey Ukolov (zezic51@yandex.ru)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[macro_use]
extern crate nih_plug;

use atomic_float::AtomicF32;
use nih_plug::{
    formatters, util, Buffer, BufferConfig, BusConfig, Editor, Plugin, ProcessContext,
    ProcessStatus, Vst3Plugin,
};
use nih_plug::{FloatParam, Param, Params, Range, Smoother, SmoothingStyle};
use nih_plug_egui::egui::plot::{Polygon, Value, Values};
use nih_plug_egui::egui::{vec2, Color32, Sense, Shape, Stroke, Vec2};
use nih_plug_egui::{create_egui_editor, egui, AtomicCell};
use std::collections::{vec_deque, VecDeque};
use std::f32::consts::TAU;
use std::pin::Pin;
use std::sync::Arc;

/// This is mostly identical to the gain example, minus some fluff, and with a GUI.
struct Vu {
    params: Pin<Arc<VuParams>>,
    editor_size: Arc<AtomicCell<(u32, u32)>>,

    history: [VecDeque<f32>; 2],
    sums: Arc<AtomicCell<(f32, f32)>>,
    last_rots: Arc<AtomicCell<(f32, f32)>>,
    history_len: Arc<AtomicCell<f32>>,

    last_inst: Arc<AtomicCell<quanta::Instant>>,
    last_fps: Arc<AtomicCell<quanta::Instant>>,
    counter: Arc<AtomicCell<usize>>,
    fps: Arc<AtomicCell<usize>>,
}

#[derive(Params)]
struct VuParams {
    #[id = "trim"]
    pub trim: FloatParam,
}

impl Default for Vu {
    fn default() -> Self {
        Self {
            params: Arc::pin(VuParams::default()),
            editor_size: Arc::new(AtomicCell::new((640, 254))),

            history: [VecDeque::new(), VecDeque::new()],
            sums: Arc::new(AtomicCell::new((0.0, 0.0))),
            last_rots: Arc::new(AtomicCell::new((0.0, 0.0))),
            history_len: Arc::new(AtomicCell::new(256.0)),

            last_inst: Arc::new(AtomicCell::new(quanta::Instant::now())),
            last_fps: Arc::new(AtomicCell::new(quanta::Instant::now())),
            counter: Arc::new(AtomicCell::new(0)),
            fps: Arc::new(AtomicCell::new(0)),
        }
    }
}

impl Default for VuParams {
    fn default() -> Self {
        Self {
            trim: FloatParam {
                value: 0.0,
                smoothed: Smoother::new(SmoothingStyle::Linear(50.0)),
                value_changed: None,
                range: Range::Linear {
                    min: -30.0,
                    max: 30.0,
                },
                name: "Trim",
                unit: " dB",
                value_to_string: formatters::f32_rounded(2),
                string_to_value: None,
            },
        }
    }
}

impl Plugin for Vu {
    const NAME: &'static str = "Vu";
    const VENDOR: &'static str = "Sergey Ukolov";
    const URL: &'static str = "https://github.com/zezic";
    const EMAIL: &'static str = "zezic51@yandex.ru";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_NUM_INPUTS: u32 = 2;
    const DEFAULT_NUM_OUTPUTS: u32 = 2;

    const ACCEPTS_MIDI: bool = false;

    fn params(&self) -> Pin<&dyn Params> {
        self.params.as_ref()
    }

    fn editor(&self) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let sums = self.sums.clone();
        let history_len = self.history_len.clone();
        let last_rots = self.last_rots.clone();

        let counter = self.counter.clone();
        let fps = self.fps.clone();
        let last_fps = self.last_fps.clone();
        let last_inst = self.last_inst.clone();

        create_egui_editor(
            self.editor_size.clone(),
            (),
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // Stats
                    counter.store(counter.load() + 1);
                    let now = quanta::Instant::now();
                    let dur = now.duration_since(last_inst.load());
                    let maybe_sec = now.duration_since(last_fps.load());
                    if maybe_sec > std::time::Duration::from_secs(1) {
                        fps.store(counter.load());
                        counter.store(0);
                        last_fps.store(now);
                    }
                    last_inst.store(now);
                    // ui.allocate_space(egui::Vec2::splat(3.0));
                    // ui.label(format!("Frame (us): {}", dur.as_micros()));
                    // ui.label(format!("FPS: {}", fps.load()));

                    // Hands
                    let mut new_rots = [0.0, 0.0];
                    let old_rots = last_rots.load();
                    let old_rots = [old_rots.0, old_rots.1];
                    let sums = sums.load();
                    let sums = [sums.0, sums.1];

                    let size = ui.available_size();
                    let (response, painter) = ui.allocate_painter(size, Sense::hover());

                    for chan_idx in 0..2 {
                        let motion_blur_color =
                            Color32::from_rgb(0x80, 0x80, 0x80);

                        let hand_color = Color32::from_gray(0xFF);
                        let stroke_h = Stroke::new(1.0, hand_color);

                        // True RMS
                        // let rms = ((1.0 / history_len.load()) * sums[chan_idx]).sqrt();

                        // Fancy RMS
                        let rms = (sums[chan_idx] / history_len.load()).sqrt();

                        let peak = 1.0 + rms.log10();
                        let range_radians = 47.0 * std::f32::consts::PI / 180.0;
                        let rotation_radians = (peak * range_radians).clamp(-range_radians, range_radians);
                        let rect = response.rect;
                        let mut c = rect.center();
                        c.x = rect.width() / 4.0 + rect.width() / 2.0 * (chan_idx as f32);
                        c.y = c.y + rect.height() * 0.35;

                        let hand_len = rect.height() * 0.72;
                        new_rots[chan_idx] = rotation_radians;
                        let old_rot = old_rots[chan_idx];

                        let new_hand_pin =
                            c + hand_len * Vec2::angled(rotation_radians - std::f32::consts::FRAC_PI_2);
                        let old_hand_pin =
                            c + hand_len * Vec2::angled(old_rot - std::f32::consts::FRAC_PI_2);

                        // Motion blur
                        painter.add(Shape::convex_polygon(
                            vec![c, new_hand_pin, old_hand_pin],
                            motion_blur_color,
                            Stroke::new(0.0, motion_blur_color),
                        ));

                        // Hand
                        painter.line_segment([c, new_hand_pin], stroke_h);
                    }

                    last_rots.store((new_rots[0], new_rots[1]));
                });
            },
        )
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl ProcessContext,
    ) -> bool {
        // TODO: How do you tie this exponential decay to an actual time span?
        // let window_len = (buffer_config.sample_rate as f32 * 0.3) as usize; // how much samples fit in 300 ms
        let window_len = (buffer_config.sample_rate as f32 * 0.150) as usize; // how much samples fit in 150 ms
        for history in &mut self.history {
            history.resize(window_len, 0.0);
        }
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        // let gain = self.params.trim.smoothed.next();
        let channels = buffer.as_raw();
        let buf_size = channels[0].len();

        let mut new_sums = [0.0, 0.0];

        for (chan_idx, channel) in channels.iter().enumerate() {
            let history = &mut self.history[chan_idx];
            let size_of_slice_to_push = buf_size.min(history.len());
            let slice = &channel[buf_size - size_of_slice_to_push..buf_size];
            history.rotate_left(size_of_slice_to_push);
            let _ = history.split_off(history.len() - size_of_slice_to_push);
            history.extend(slice);
            let sum: f32 = history.iter().map(|x| x.powf(2.0)).sum::<f32>();

            self.history_len.store(history.len() as f32);

            new_sums[chan_idx] = sum;
        }

        self.sums.store((new_sums[0], new_sums[1]));

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for Vu {
    const VST3_CLASS_ID: [u8; 16] = *b"ItsSuperSmoothVu";
    const VST3_CATEGORIES: &'static str = "Fx|Analyzer";
}

nih_export_vst3!(Vu);
