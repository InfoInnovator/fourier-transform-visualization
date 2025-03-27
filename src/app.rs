use core::{f64, fmt};

use egui::{ComboBox, DragValue, TopBottomPanel};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, PlotPoints};
use num_complex::Complex;

#[derive(Default)]
pub struct TemplateApp {
    functions: Vec<InputData>,
    num_samples: usize,
    input_signal_range: f64,
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            num_samples: 1000,
            input_signal_range: 3.14,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
struct InputData {
    function: PeriodicFunction,
    amplitude: f64,
    frequency: f64,
    y_shift: f64,
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
                    .column(Column::exact(100.0))
                    .column(Column::exact(100.0))
                    .column(Column::exact(100.0));
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
                        header.col(|ui| {
                            ui.heading("Y Shift");
                        });
                    })
                    .body(|body| {
                        body.rows(25.0, self.functions.len(), |mut row| {
                            let index = row.index();
                            let function = &mut self.functions[index];

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
                            row.col(|ui| {
                                ui.add(DragValue::new(&mut function.y_shift).speed(0.1));
                            });
                        });
                    });

                if ui.button("Add function").clicked() {
                    self.functions.push(InputData {
                        function: PeriodicFunction::Sin,
                        amplitude: 1.0,
                        frequency: 1.0,
                        y_shift: 0.0,
                    });
                }

                TopBottomPanel::bottom("bottom_controls").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Number of samples");
                        ui.add(DragValue::new(&mut self.num_samples).speed(1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Input signal range");
                        ui.add(
                            DragValue::new(&mut self.input_signal_range)
                                .speed(0.1)
                                .range(0.0..=100.0),
                        );
                    });
                });
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
                            move |x| {
                                applied_function(x * function.frequency) * function.amplitude
                                    + function.y_shift
                            },
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
                    let input = get_combined_wave(
                        self.functions.clone(),
                        self.num_samples,
                        self.input_signal_range,
                    );

                    plot_ui.line(Line::new(
                        input
                            .iter()
                            .map(|(x, y)| [*x, y.re])
                            .collect::<PlotPoints>(),
                    ));
                });

            egui_plot::Plot::new("Frequency Plot")
                .link_axis("axes_group", [true, true])
                .clamp_grid(true)
                .link_cursor("cursor_link", [true, true].into())
                .show(ui, |plot_ui| {
                    let mut input = get_combined_wave(
                        self.functions.clone(),
                        self.num_samples,
                        self.input_signal_range,
                    );

                    let mut new_input = input.iter().map(|(_, y)| *y).collect::<Vec<_>>();
                    fft(&mut new_input);

                    input = input
                        .iter()
                        .zip(new_input.iter())
                        .map(|((x, _), y)| (*x, *y))
                        .collect();

                    plot_ui.line(Line::new(
                        input
                            .iter()
                            .map(|(x, y)| [*x, y.re])
                            .collect::<PlotPoints>(),
                    ));
                });
        });
    }
}

fn get_combined_wave(
    functions: Vec<InputData>,
    num_samples: usize,
    input_signal_range: f64,
) -> Vec<(f64, Complex<f64>)> {
    let step_size = input_signal_range / num_samples as f64;
    let mut input = Vec::new();

    let mut i = 0.0;
    while i < input_signal_range {
        let mut sum = 0.0;
        for function in functions.iter() {
            let applied_function = match function.function {
                PeriodicFunction::Sin => f64::sin,
                PeriodicFunction::Cos => f64::cos,
            };
            sum += applied_function(i * function.frequency) * function.amplitude + function.y_shift;
        }
        input.push((i, sum.into()));
        i += step_size;
    }

    input
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
