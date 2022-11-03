use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    Arc, Mutex,
};

use eframe::{egui, epaint::ColorImage};
use egui_extras::RetainedImage;
use image::RgbImage;
use poll_promise::Promise;

use crate::{camera::Camera, util::ProgressBarWrapper, world::World, RaytraceParams};

pub fn run_gui(params: RaytraceParams, world: World, camera: Camera) {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Raytracer",
        options,
        Box::new(move |_cc| Box::new(RaytracerApp::new(params, world, camera))),
    );
}

struct RaytracerApp {
    render_action: Option<RenderAction>,
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

        let progress: Box<dyn ProgressBarWrapper> =
            Box::new(Arc::clone(&render_action.progress));

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
}

impl eframe::App for RaytracerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            if let Some(render_action) = self.render_action.as_mut() {
                match render_action.image_promise.ready() {
                    None => {
                        // Update progress bar
                        let progress_bar =
                            egui::ProgressBar::new(render_action.progress.percentage())
                                .show_percentage()
                                .animate(true);
                        ui.add(progress_bar);

                        // Update immediate image from progress struct
                        let mut immediate_image =
                            render_action.progress.immediate_image.lock().unwrap();
                        if let Some(immediate_image) = immediate_image.take() {
                            render_action.immediate_image = Some(RetainedImage::from_color_image(
                                "immediate_image",
                                immediate_image,
                            ));
                        }

                        // Show immediate image
                        if let Some(immediate_image) = render_action.immediate_image.as_ref() {
                            immediate_image.show_scaled(ui, 1.5);
                        }
                    }
                    Some(image) => {
                        image.show_scaled(ui, 1.5);
                    }
                }
            }

            if ui.button("Render").clicked() {
                self.start_render(ctx);
            }

            self.num_draws += 1;
            ui.label(format!("Drawn {} times.", self.num_draws));
        });
    }
}
