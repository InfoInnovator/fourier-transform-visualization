use core::{f64, fmt};

use egui::{ComboBox, DragValue};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, PlotPoints};
use num_complex::{Complex, ComplexFloat};

#[derive(Default)]
pub struct TemplateApp {
    functions: Vec<InputData>,
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

#[derive(Clone)]
struct InputData {
    function: PeriodicFunction,
    amplitude: f64,
    frequency: f64,
}

#[derive(Clone, PartialEq)]
enum PeriodicFunction {
    Sin,
    Cos,
}

impl fmt::Display for PeriodicFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeriodicFunction::Sin => write!(f, "Sin"),
            PeriodicFunction::Cos => write!(f, "Cos"),
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel")
            .min_width(ctx.available_rect().size().x * 0.2)
            .show(ctx, |ui| {
                let wave_table = TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto());
                wave_table
                    .header(25.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Function");
                        });
                        header.col(|ui| {
                            ui.heading("Amplitude");
                        });
                        header.col(|ui| {
                            ui.heading("Frequency");
                        });
                    })
                    .body(|body| {
                        body.rows(25.0, self.functions.len(), |mut row| {
                            let function = &mut self.functions[row.index()];

                            row.col(|ui| {
                                ComboBox::from_id_salt("function")
                                    .selected_text(format!("{}", function.function))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut function.function,
                                            PeriodicFunction::Sin,
                                            "Sin",
                                        );
                                        ui.selectable_value(
                                            &mut function.function,
                                            PeriodicFunction::Cos,
                                            "Cos",
                                        );
                                    });
                            });
                            row.col(|ui| {
                                ui.add(DragValue::new(&mut function.amplitude).speed(0.1));
                            });
                            row.col(|ui| {
                                ui.add(DragValue::new(&mut function.frequency).speed(0.1));
                            });
                        });
                    });

                if ui.button("Add function").clicked() {
                    self.functions.push(InputData {
                        function: PeriodicFunction::Sin,
                        amplitude: 1.0,
                        frequency: 1.0,
                    });
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui_plot::Plot::new("Time Plot")
                .height(ui.available_height() / 3.0)
                .link_cursor("cursor_link", [true, true].into())
                .link_axis("axes_group", [true, true])
                .show(ui, |plot_ui| {
                    let functions = self.functions.clone();

                    for function in functions {
                        let applied_function = match function.function {
                            PeriodicFunction::Sin => f64::sin,
                            PeriodicFunction::Cos => f64::cos,
                        };
                        plot_ui.line(Line::new(PlotPoints::from_explicit_callback(
                            move |x| applied_function(x * function.frequency) * function.amplitude,
                            f64::NEG_INFINITY..=f64::INFINITY,
                            10000,
                        )));
                    }
                });

            egui_plot::Plot::new("Combined Wave")
                .height(ui.available_height() / 2.0)
                .link_cursor("cursor_link", [true, true].into())
                .link_axis("axes_group", [true, true])
                .show(ui, |plot_ui| {
                    let mut input = Vec::new();
                    let num_samples = i32::pow(2, 15) as usize;
                    let step_size = 100.0 / num_samples as f64;

                    let mut i = 0.0;
                    while i < 10.0 {
                        let mut sum = 0.0;
                        for function in self.functions.iter() {
                            let applied_function = match function.function {
                                PeriodicFunction::Sin => f64::sin,
                                PeriodicFunction::Cos => f64::cos,
                            };
                            sum += applied_function(i * function.frequency) * function.amplitude;
                        }
                        input.push(sum);
                        i += step_size;
                    }

                    let input_len = input.len() as f64;

                    plot_ui.line(Line::new(PlotPoints::from_explicit_callback(
                        move |x| input[(x / step_size) as usize].re(),
                        0.0..=10.0,
                        input_len as usize,
                    )));
                });

            egui_plot::Plot::new("Frequency Plot")
                .link_axis("axes_group", [true, true])
                .clamp_grid(true)
                .link_cursor("cursor_link", [true, true].into())
                .show(ui, |plot_ui| {
                    let mut input: Vec<Complex<f64>> = Vec::new();
                    let num_samples = i32::pow(2, 15) as usize;
                    let step_size = 100.0 / num_samples as f64;

                    let mut i = 0.0;
                    while i < 10.0 {
                        let mut sum = 0.0;
                        for function in self.functions.iter() {
                            let applied_function = match function.function {
                                PeriodicFunction::Sin => f64::sin,
                                PeriodicFunction::Cos => f64::cos,
                            };
                            sum += applied_function(i * function.frequency) * function.amplitude;
                        }
                        input.push(sum.into());
                        i += step_size;
                    }

                    fft(&mut input);

                    let input_len = input.len() as f64;
                    plot_ui.line(Line::new(PlotPoints::from_explicit_callback(
                        move |x| input[(x / step_size) as usize].re() * 0.001,
                        0.0..=10.0,
                        input_len as usize,
                    )));
                });
        });
    }
}

fn fft(input: &mut [Complex<f64>]) {
    let n = input.len();
    if n <= 1 {
        return;
    }

    let mut even: Vec<Complex<f64>> = input.iter().step_by(2).copied().collect();
    let mut odd: Vec<Complex<f64>> = input.iter().skip(1).step_by(2).copied().collect();

    fft(&mut even);
    fft(&mut odd);

    for i in 0..n / 2 {
        let t =
            Complex::from_polar(1.0, -2.0 * std::f64::consts::PI * i as f64 / n as f64) * odd[i];
        input[i] = even[i] + t;
        input[i + n / 2] = even[i] - t;
    }
}
