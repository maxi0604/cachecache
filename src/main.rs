use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{error::Error, thread};

use gtk::gio::{ApplicationFlags, ApplicationCommandLine};
use gtk::glib::{MainContext, Priority};
use gtk::{prelude::*, ApplicationWindow, ScrolledWindow, PolicyType, Button, Orientation, Label, Align, Separator};
use gtk::{Application, glib};
use sim::{CacheEntry, CacheStats, CacheDesc};
use glib::clone;

mod sim;

const APP_ID: &str = "com.github.maxi0604.CacheCache";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_command_line(build_ui);

    app.run()
}

enum SimulationCommunication {
    Success((CacheLineVec, CacheDesc, Vec<u64>, CacheStats)),
    Failure,
    Run
}

fn build_ui(app: &Application, command_line: &ApplicationCommandLine) -> i32 {
    let arguments: Vec<OsString> = command_line.arguments();
    let mut path_buf = PathBuf::new();
    if let Some(os_string) = arguments.get(1) {
        path_buf.push(Path::new(os_string));
    }

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .min_content_width(260)
        .vexpand(true)
        .build();

    let separator_top = Separator::new(Orientation::Horizontal);
    let separator_bottom = Separator::new(Orientation::Horizontal);

    let simulate_button = Button::builder()
        .label("Simulate")
        .margin_end(10)
        .margin_top(10)
        .margin_start(10)
        .margin_bottom(10)
        .build();

    let stats_showcase = Label::builder().visible(false).build();

    let (sim_sender, sim_receiver) = MainContext::channel(Priority::default());

    simulate_button.connect_clicked(move |_| {
        let sender = sim_sender.clone();
        let path_buf = path_buf.clone();
        thread::spawn(move || {
            sender.send(SimulationCommunication::Run).expect("Could not send through channel");
            
            match run_sim(&path_buf) {
                Ok(result) => {
                    sender.send(SimulationCommunication::Success(result)).expect("Could not send through channel");
                },
                Err(err) => {
                    eprintln!("run_sim: {}", err);
                    sender.send(SimulationCommunication::Failure).expect("Could not send through channel");
                }
            }

        });
    });

    let (stats_sender, stats_receiver) = MainContext::channel(Priority::default());

    sim_receiver.attach(None, clone!(@weak simulate_button, @weak scrolled_window => @default-return Continue(false),
        move |result| {
            let stats_sender = stats_sender.clone();
            match result {
                SimulationCommunication::Success((lines, cache, addrs, stats)) => {
                    simulate_button.set_sensitive(true);
                    
                    let grid = gtk::Grid::builder()
                        .margin_end(10)
                        .margin_top(10)
                        .margin_start(10)
                        .margin_bottom(10)
                        .column_spacing(10)
                        .build();

                    for (i, line) in lines.iter().enumerate() {
                        let line_index: i32 = (i as i32).try_into().unwrap();

                        let line_label = Label::builder().label(format!("{}", line_index + 1)).halign(Align::End).build();

                        grid.attach(&line_label, 0, line_index, 2, 1);

                        if line.is_empty() {
                            let label = Label::builder().label("-").build();
                            grid.attach(&label, 2, line_index, 1, 1);
                        } else {
                            let mut column_index: i32 = 2;

                            for entry in line.iter() {
                                let label = Label::builder().label(format!("{} ({})", entry.tag(), entry.entered())).build();
                                grid.attach(&label, column_index, line_index, 1, 1);
                                column_index += 1;
                            }
                        }
                    }

                    scrolled_window.set_child(Some(&grid));
                    stats_sender.send(Some((cache, addrs, stats))).expect("Could not send through stats channel");
                },
                SimulationCommunication::Failure => {
                    simulate_button.set_sensitive(true);
                },
                SimulationCommunication::Run => {
                    simulate_button.set_sensitive(false);
                }
            }
            Continue(true)
        }
    ));

    let container_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    stats_receiver.attach(None, clone!(@weak stats_showcase => @default-return Continue(false), 
        move |stats: Option<(CacheDesc, Vec<u64>, CacheStats)>| {
            match stats {
                Some((_cache, addrs, stats)) => {
                    stats_showcase.set_label(format!("Hits: {1}/{0}. Misses: {2}/{0}. Evictions: {3}/{0}", addrs.len(), stats.hits(), stats.misses(), stats.evictions()).as_str());
                    stats_showcase.set_visible(true);
                }
                None => {
                    stats_showcase.set_visible(false);
                }
            }
            Continue(true)
        }
    ));

    container_box.append(&simulate_button);
    container_box.append(&separator_top);
    container_box.append(&scrolled_window);
    container_box.append(&separator_bottom);
    container_box.append(&stats_showcase);

    let window = ApplicationWindow::builder()
        .title("CacheCache")
        .application(app)
        .default_height(400)
        .default_width(500)
        .child(&container_box)
        .build();

    window.present();
    0
}

type CacheLineVec = Vec<Vec<CacheEntry>>;

fn run_sim(path: &PathBuf) -> Result<(CacheLineVec, CacheDesc, Vec<u64>, sim::CacheStats), Box<dyn Error>> {
    let (cache, addrs) = sim::read(path)?;

    let (lines, stats) = sim::simulate(&cache, &addrs);

    Ok((lines, cache, addrs, stats))
}
