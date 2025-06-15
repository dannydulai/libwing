mod utils;
use utils::Args;
use std::sync::{Arc, RwLock};
use std::thread;
use eframe::egui::{self, vec2, Rect, Color32, RichText, FontId, Pos2};
use libwing::{WingConsole, Meter};

// Number of channels to request meters for max 40
const CHANNEL_COUNT: u8 = 16;

fn main() -> Result<(), libwing::Error> {
    let mut args = Args::new(r#"
Usage: wingmeters [-h host]

   -h host : IP address or hostname of Wing mixer. Default is to discover and connect to the first mixer found.
"#);
    let mut host = None;
    if args.has_next() && args.next() == "-h" { host = Some(args.next()); }

    let options = eframe::NativeOptions {
        vsync: true,
        // initial_window_size: Some(vec2(800.0, 400.0)),
        ..Default::default()
    };

    // Request meters for first 32 channels
    let meters: Vec<Meter> = (0..CHANNEL_COUNT).map(Meter::Channel).collect();


    let mut wing = WingConsole::connect(host.as_deref())?;
    wing.request_meter(&meters)?;

    eframe::run_native(
        "Wing Meters",
        options,
        Box::new(|_cc| Box::new(WingMetersApp::new(wing))),
    ).unwrap();

    Ok(())
}

struct WingMetersApp {
    meters: Arc<RwLock<Vec<f32>>>,
}

impl WingMetersApp {
    fn new(mut wing: WingConsole) -> Self {
        let meters = Arc::new(RwLock::new(vec![0.0; (CHANNEL_COUNT * 2).into()]));
        let m = meters.clone();

        let _ = thread::spawn(move || {
            loop {
                if let Ok((_, values)) = wing.read_meters() {
                    let mut vals = m.write().unwrap();
                    for i in 0..CHANNEL_COUNT.into() {
                        vals[2*i]   = ((values[i*8 + 0] as f32 / 256.0 + 60.0) / 60.0).clamp(0.0, 1.0);
                        vals[2*i+1] = ((values[i*8 + 2] as f32 / 256.0 + 60.0) / 60.0).clamp(0.0, 1.0);
                    }
                }
            }
        });

        Self {
            meters,
        }
    }
}

impl eframe::App for WingMetersApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let vals = self.meters.read().unwrap();
            let num_cols = vals.len()/2;

            ui.columns(num_cols, |col| {
                for i in 0..num_cols {
                    let left = vals[2*i];
                    let right = vals[2*i+1];

                    col[i].vertical(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("CH{}", i + 1)).font(FontId::proportional(10.0)));
                        });
                        ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                        ui.columns(2, |c| {
                            c[0].vertical_centered(|ui| {
                                ui.label(RichText::new("L").font(FontId::proportional(10.0)));
                            });
                            c[1].vertical_centered(|ui| {
                                ui.label(RichText::new("R").font(FontId::proportional(10.0)));
                            });
                        });

                        ui.add_space(2.0);

                        let meter_height = ui.available_height();
                        let (_id, rect) = ui.allocate_space(vec2(ui.available_width(), meter_height));

                        // Draw meter value
                        let color = if left > 0.9 {
                            Color32::RED
                        } else if left > 0.7 {
                            Color32::YELLOW
                        } else {
                            Color32::GREEN
                        };

                        // bg left
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                Pos2::new(rect.left(), rect.bottom() - meter_height),
                                vec2(rect.width()/2.0-1.0, meter_height),
                            ),
                            0.0,
                            Color32::from_gray(64),
                        );
                        // fg left
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                Pos2::new(rect.left(), rect.bottom() - meter_height * left),
                                vec2(rect.width()/2.0-1.0, meter_height * left),
                            ),
                            0.0,
                            color,
                        );

                        // bg right
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                Pos2::new(rect.left() + rect.width()/2.0+1.0, rect.bottom() - meter_height),
                                rect.max,
                            ),
                            0.0,
                            Color32::from_gray(64),
                        );
                        // fg right
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                Pos2::new(rect.left() + rect.width()/2.0+1.0, rect.bottom() - meter_height * right),
                                rect.max,
                            ),
                            0.0,
                            color,
                        );
                    });
                }
            });
        });

        // Request continuous updates
        ctx.request_repaint();
    }
}
