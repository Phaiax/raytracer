use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    Arc,
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
    image_promise: Option<Promise<RetainedImage>>,
    progress: Option<Arc<ProgressInfo>>,
    num_draws: u32,
    params: RaytraceParams,
    world: Arc<World>,
    camera: Camera,
}

struct ProgressInfo {
    len: AtomicU64,
    progress: AtomicU64,
    finished: AtomicBool,
    ctx: egui::Context,
}

impl ProgressInfo {
    fn new(ctx: egui::Context) -> Self {
        ProgressInfo {
            len: 0.into(),
            progress: 0.into(),
            finished: false.into(),
            ctx,
        }
    }
}

impl ProgressBarWrapper for Arc<ProgressInfo> {
    fn set_length(&self, len: u64) {
        self.len.store(len, std::sync::atomic::Ordering::Relaxed);
        self.ctx.request_repaint();
    }

    fn inc(&self, delta: u64) {
        self.progress
            .fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
        self.ctx.request_repaint();
    }

    fn finish(&self) {
        self.finished
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.ctx.request_repaint();
    }
}

impl RaytracerApp {
    fn new(params: RaytraceParams, world: World, camera: Camera) -> Self {
        RaytracerApp {
            image_promise: None,
            progress: None,
            params,
            world: Arc::new(world),
            camera,
            num_draws: 0,
        }
    }
}

impl eframe::App for RaytracerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            if let Some(image_promise) = self.image_promise.as_ref() {
                match image_promise.ready() {
                    None => {
                        ui.spinner();
                        let progress = self.progress.as_ref().unwrap();
                        let progress = progress.progress.load(std::sync::atomic::Ordering::Relaxed)
                            as f32
                            / progress.len.load(std::sync::atomic::Ordering::Relaxed) as f32;
                        let progress_bar = egui::ProgressBar::new(progress)
                            .show_percentage()
                            .animate(true);
                        ui.add(progress_bar);
                    }
                    Some(image) => {
                        image.show_scaled(ui, 1.5);
                    }
                }
            } else {
                if ui.button("Render").clicked() {
                    self.image_promise.get_or_insert_with(|| {
                        let (sender, promise) = Promise::new();

                        self.progress = Some(Arc::new(ProgressInfo::new(ctx.clone())));
                        let progress: Box<dyn ProgressBarWrapper> =
                            Box::new(Arc::clone(&self.progress.as_ref().unwrap()));
                        let params = self.params.clone();
                        let world = Arc::clone(&self.world);
                        let camera = self.camera.clone();

                        rayon::spawn(move || {
                            let img = crate::render(&params, &world, &camera, &progress);
                            let img = ColorImage::from_rgba_unmultiplied(
                                [img.width() as usize, img.height() as usize],
                                img.as_flat_samples().samples,
                            );
                            sender.send(RetainedImage::from_color_image("rendered_image", img));
                        });
                        promise
                    });
                }
            }
            self.num_draws += 1;
            ui.label(format!("Drawn {} times.", self.num_draws));
        });
    }
}
