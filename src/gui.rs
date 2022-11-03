use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    Arc, Mutex,
};

use eframe::{
    egui,
    epaint::{ColorImage, Pos2, Vec2},
    NativeOptions,
};
use egui_extras::RetainedImage;
use image::RgbImage;
use poll_promise::Promise;

use crate::{camera::Camera, util::ProgressBarWrapper, world::World, RaytraceParams};

pub fn run_gui(params: RaytraceParams, world: World, camera: Camera) {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(2000.0, 1300.0)),
        initial_window_pos: Some(Pos2::new(600.0, 300.0)),
        ..NativeOptions::default()
    };
    eframe::run_native(
        "Raytracer",
        options,
        Box::new(move |_cc| Box::new(RaytracerApp::new(params, world, camera))),
    );
}

struct RaytracerApp {
    render_action: Option<RenderAction>,
    final_render: Option<RetainedImage>,
    num_draws: u32,
    params: RaytraceParams,
    world: Arc<World>,
    camera: Camera,
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
    fn new(params: RaytraceParams, world: World, camera: Camera) -> Self {
        RaytracerApp {
            render_action: None,
            final_render: None,
            params,
            world: Arc::new(world),
            camera,
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

        let params = self.params.clone();
        let world = Arc::clone(&self.world);
        let camera = self.camera.clone();
        let stop = Arc::clone(&render_action.stop);

        let progress: Box<dyn ProgressBarWrapper> = Box::new(Arc::clone(&render_action.progress));

        rayon::spawn(move || {
            let img = crate::render_live(&params, &world, &camera, &progress, stop);
            let img = ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_flat_samples().samples,
            );
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
            self.final_render = Some(image);
        }
    }
}

impl eframe::App for RaytracerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            // .min_width(600.0)
            // .max_width(600.0)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Raytracer");
                ui.label("Hello World!");
                if ui.button("Render").clicked() {
                    self.start_render(ctx);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_action
                .as_mut()
                .map(|ra| {
                    ra.take_immediate_image();
                    ra.immediate_image.as_ref()
                })
                .flatten()
                .or_else(|| self.final_render.as_ref())
                .map(|i| i.show_scaled(ui, 1.5));
        });
    }
}
