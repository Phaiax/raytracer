use std::{
    ops::RangeInclusive,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
        Arc, Mutex,
    },
};

use eframe::{
    egui::{self, Context, Id, Slider, Ui},
    epaint::{ColorImage, Pos2, Vec2},
    NativeOptions,
};
use egui_extras::RetainedImage;
use image::RgbImage;
use poll_promise::Promise;

use crate::{camera::CameraBuilder, util::ProgressBarWrapper, world::World, RaytraceParams};

pub fn run_gui(params: RaytraceParams, world: World, camerabuilder: CameraBuilder) {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(2000.0, 1300.0)),
        initial_window_pos: Some(Pos2::new(600.0, 300.0)),
        ..NativeOptions::default()
    };
    eframe::run_native(
        "Raytracer",
        options,
        Box::new(move |_cc| Box::new(RaytracerApp::new(params, world, camerabuilder))),
    );
}

struct RaytracerApp {
    startup_done: bool,
    render_action: Option<RenderAction>,
    final_render: Option<RetainedImage>,
    num_draws: u32,
    params: RaytraceParams,
    world: Arc<World>,
    camerabuilder: CameraBuilder,
}

struct RenderAction {
    image_promise: Promise<RetainedImage>,
    immediate_image: Option<RetainedImage>,
    progress: Arc<ProgressInfo>,
    stop: Arc<AtomicBool>,
}

impl RenderAction {
    fn take_immediate_image(&mut self) {
        let mut immediate_image = self.progress.immediate_image.lock().unwrap();
        if let Some(immediate_image) = immediate_image.take() {
            self.immediate_image = Some(RetainedImage::from_color_image(
                "immediate_image",
                immediate_image,
            ));
        }
    }
}

struct ProgressInfo {
    current: AtomicU64,
    len: AtomicU64,
    finished: AtomicBool,
    immediate_image: Mutex<Option<ColorImage>>,
    ctx: egui::Context,
}

impl ProgressInfo {
    fn new(ctx: egui::Context) -> Self {
        ProgressInfo {
            current: 0.into(),
            len: 0.into(),
            finished: false.into(),
            immediate_image: Mutex::new(None),
            ctx,
        }
    }

    fn percentage(&self) -> f32 {
        self.current.load(Relaxed) as f32 / self.len.load(Relaxed) as f32
    }
}

impl ProgressBarWrapper for Arc<ProgressInfo> {
    fn set_length(&self, len: u64) {
        self.len.store(len, Relaxed);
        self.ctx.request_repaint();
    }

    fn inc(&self, delta: u64, get_immediate_image: &dyn Fn() -> ColorImage) {
        self.current.fetch_add(delta, Relaxed);
        *self.immediate_image.lock().unwrap() = Some(get_immediate_image());
        self.ctx.request_repaint();
    }

    fn finish(&self) {
        self.finished.store(true, Relaxed);
        self.ctx.request_repaint();
    }
}

impl RaytracerApp {
    fn new(params: RaytraceParams, world: World, camerabuilder: CameraBuilder) -> Self {
        RaytracerApp {
            startup_done: false,
            render_action: None,
            final_render: None,
            params,
            world: Arc::new(world),
            camerabuilder,
            num_draws: 0,
        }
    }

    fn start_render(&mut self, ctx: &egui::Context) {
        if let Some(old_render_action) = self.render_action.take() {
            old_render_action.stop.store(true, Relaxed);
        }

        let (sender, promise) = Promise::new();

        let render_action = RenderAction {
            image_promise: promise,
            immediate_image: None,
            progress: Arc::new(ProgressInfo::new(ctx.clone())),
            stop: Arc::new(AtomicBool::new(false)),
        };

        self.camerabuilder.aspect_ratio(self.params.aspect_ratio);

        let params = self.params.clone();
        let world = Arc::clone(&self.world);
        let camera = self.camerabuilder.build().unwrap();
        let stop = Arc::clone(&render_action.stop);

        let progress: Box<dyn ProgressBarWrapper> = Box::new(Arc::clone(&render_action.progress));

        println!("Start render with vfow={:?}", camera.vertical);
        rayon::spawn(move || {
            let img = crate::render_live(&params, &world, &camera, &progress, stop);
            let img = ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_flat_samples().samples,
            );
            println!("Done rendering with vfow={:?}", camera.vertical);
            sender.send(RetainedImage::from_color_image("rendered_image", img));
        });

        self.render_action = Some(render_action);
    }

    fn check_render_finished(&mut self) {
        let render_available = self
            .render_action
            .as_ref()
            .map(|ra| ra.image_promise.poll().is_ready())
            .unwrap_or(false);

        if render_available {
            let render_action = self.render_action.take().unwrap();
            let image = render_action.image_promise.try_take().ok().unwrap();
            println!("Get finished render");
            self.final_render = Some(image);
        }
    }

    /// Returns true on change
    fn slider<Num: eframe::emath::Numeric>(
        ui: &mut Ui,
        param: &mut Num,
        text: &str,
        suffix: &str,
        range: RangeInclusive<Num>,
        extra: impl FnOnce(Slider) -> Slider,
    ) -> bool {
        let param_orig: Num = *param;
        let mut param_mut = param_orig;
        let mut slider = egui::Slider::new(&mut param_mut, range)
            .text(text)
            .suffix(suffix);
        slider = extra(slider);
        ui.add_space(5.0);
        ui.add(slider);
        ui.add_space(5.0);
        if param_mut != param_orig {
            *param = param_mut;
            true
        } else {
            false
        }
    }
}

impl eframe::App for RaytracerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.startup_done {
            self.start_render(ctx);
            self.startup_done = true;
        }

        self.num_draws += 1;

        let progressbar = self
            .render_action
            .as_ref()
            .map(|ra| (ra.progress.percentage(), ra.progress.finished.load(Relaxed)))
            .map(|(percentage, finished)| {
                egui::ProgressBar::new(percentage)
                    .show_percentage()
                    .animate(finished)
            })
            .unwrap_or_else(|| egui::ProgressBar::new(1.0).show_percentage());

        self.check_render_finished();

        egui::TopBottomPanel::bottom("status_bar")
            .default_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Drawn {} times.", self.num_draws));
                    ui.add(progressbar);
                    ui.allocate_space(ui.available_size());
                });
            });

        egui::SidePanel::left("control_panel")
            .resizable(false)
            .min_width(600.0)
            .max_width(600.0)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Raytracer");
                if ui.button("Render").clicked() {
                    self.start_render(ctx);
                }
                ui.style_mut().spacing.slider_width = 400.0;

                let mut changed = false;

                ui.heading("Camera");
                changed |= Self::slider(
                    ui,
                    self.camerabuilder.vfov.as_mut().unwrap(),
                    "Vertical Field of View",
                    "°",
                    30.0..=180.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    self.camerabuilder.aperture.as_mut().unwrap(),
                    "Aperture",
                    "",
                    0.001..=1.0,
                    |s: egui::Slider| s.logarithmic(true),
                );
                changed |= Self::slider(
                    ui,
                    self.camerabuilder.focus_dist.as_mut().unwrap(),
                    "Focus Distance",
                    "",
                    0.1..=100.0,
                    |s| s,
                );
                ui.label("Look At Position");
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.lookat.as_mut().unwrap().x,
                    "Lookat X",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.lookat.as_mut().unwrap().y,
                    "Lookat Y",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.lookat.as_mut().unwrap().z,
                    "Lookat Z",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                ui.label("Look From Position");
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.lookfrom.as_mut().unwrap().x,
                    "Lookfrom X",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.lookfrom.as_mut().unwrap().y,
                    "Lookfrom Y",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.lookfrom.as_mut().unwrap().z,
                    "Lookfrom Z",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );

                ui.label("Vertical Up Direction");
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.vup.as_mut().unwrap().x,
                    "Vup X",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.vup.as_mut().unwrap().y,
                    "Vup Y",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.camerabuilder.vup.as_mut().unwrap().z,
                    "Vup Z",
                    "",
                    -10.0..=10.0,
                    |s| s,
                );

                ui.heading("Rendering");
                changed |= Self::slider(
                    ui,
                    &mut self.params.aspect_ratio,
                    "Aspect Ratio",
                    "",
                    0.1..=10.0,
                    |s: egui::Slider| s.logarithmic(true),
                );
                changed |= Self::slider(
                    ui,
                    &mut self.params.samples_per_pixel,
                    "Samples per pixel",
                    "",
                    1..=5000,
                    |s: egui::Slider| s.logarithmic(true),
                );
                changed |= Self::slider(
                    ui,
                    &mut self.params.max_depth,
                    "Max Depth",
                    "",
                    1..=100,
                    |s| s,
                );
                changed |= Self::slider(
                    ui,
                    &mut self.params.image_width,
                    "Image Width",
                    "px",
                    50..=3000,
                    |s| s,
                );
                if changed {
                    self.start_render(ui.ctx());
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .id_source("main_img_scroll")
                .max_width(f32::INFINITY)
                .max_height(f32::INFINITY)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let zoomstateid = Id::new("main_img_zoom");
                    let mut zoomstate = ZoomState::load(ui.ctx(), zoomstateid).unwrap_or_default();
                    zoomstate.zoom *= ui.input().zoom_delta() as f64;
                    if zoomstate.zoom > 0.99999 && zoomstate.zoom < 1.000001 {
                        zoomstate.zoom = 1.0;
                    }
                    // ui.label(format!(
                    //     "Zoom: {} (delta {})",
                    //     zoomstate.zoom,
                    //     ui.input().zoom_delta()
                    // ));
                    self.render_action
                        .as_mut()
                        .map(|ra| {
                            ra.take_immediate_image();
                            ra.immediate_image.as_ref()
                        })
                        .flatten()
                        .or_else(|| self.final_render.as_ref())
                        .map(|i| i.show_scaled(ui, zoomstate.zoom as f32));
                    zoomstate.store(ui.ctx(), zoomstateid);
                });
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ZoomState {
    pub zoom: f64,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self { zoom: 3.0 }
    }
}

impl ZoomState {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data().get_persisted(id)
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data().insert_persisted(id, self);
    }
}
