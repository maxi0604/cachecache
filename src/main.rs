use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{error::Error, thread};

use gtk::gio::{ApplicationFlags, ApplicationCommandLine, Cancellable};
use gtk::glib::{MainContext, Priority};
use gtk::{prelude::*, ScrolledWindow, PolicyType, Button, Orientation, Label, Align, Separator, FileDialog, Window, DialogError, Spinner};
use gtk::{Application, glib};
use sim::{CacheEntry, CacheStats, CacheDesc};
use glib::clone;
use window::CacheCacheWindow;

mod sim;
mod window;

const APP_ID: &str = "com.github.maxi0604.CacheCache";
type SimResult = (CacheLineVec, CacheDesc, Vec<u64>, CacheStats);

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_command_line(build_ui);

    app.run()
}

enum SimulationCommunication {
    Success(SimResult),
    Failure,
    Run
}

fn build_ui(app: &Application, command_line: &ApplicationCommandLine) -> i32 {

    let window = CacheCacheWindow::new(app);

    let arguments: Vec<OsString> = command_line.arguments();
    if let Some(os_string) = arguments.get(1) {
        let mut some_path_buf = PathBuf::new();
        some_path_buf.push(Path::new(os_string));
        window.set_path_buf(some_path_buf);
    }

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .min_content_width(150)
        .vexpand(true)
        .build();

    let separator_top = Separator::new(Orientation::Horizontal);
    let separator_bottom = Separator::new(Orientation::Horizontal);
    separator_bottom.set_visible(false);

    let file_display_label = Label::builder()
        .label("No File Selected")
        .build();
    let file_display_spinner = Spinner::builder()
        .halign(Align::End)
        .build();

    window.bind_property("path-buf", &file_display_label, "label")
        .transform_to(|_, path_buf: PathBuf| {
            if let Some(file_str) = path_buf.to_str().to_owned() {
                Some(file_str.to_value())
            } else if path_buf.is_file() {
                Some("Could not parse file name to string".to_value())
            } else {
                Some("No File Selected".to_value())
            }
        })
        .build();

    let file_display = gtk::Box::builder()
        .spacing(10)
        .margin_end(10)
        .margin_start(10)
        .margin_bottom(10)
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    file_display.append(&file_display_label);
    file_display.append(&file_display_spinner);
    
    let simulate_button = Button::builder()
        .sensitive(window.path_buf().is_file())
        .hexpand(true)
        .label("Simulate")
        .build();

    let stats_showcase = Label::builder().visible(false).build();

    stats_showcase.bind_property("visible", &separator_bottom, "visible")
        .bidirectional()
        .build();

    let (sim_sender, sim_receiver) = MainContext::channel(Priority::default());

    simulate_button.connect_clicked(clone!(@weak window => move |_| {
        let sim_sender = sim_sender.clone();
        let path_buf = window.path_buf();
        if path_buf.is_file() {
            let some_path_buf = path_buf;
            thread::spawn(move || {
                sim_sender.send(SimulationCommunication::Run).expect("Could not send through channel");

                match run_sim(&some_path_buf) {
                    Ok(result) => {
                        sim_sender.send(SimulationCommunication::Success(result)).expect("Could not send through channel");
                    },
                    Err(err) => {
                        eprintln!("run_sim: {}", err);
                        sim_sender.send(SimulationCommunication::Failure).expect("Could not send through channel");
                    }
                }

            });
        } else {
            eprintln!("no file selected");
        }
    }));

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

                        let li: u64 = (line_index as u64 / cache.n_sets()).try_into().unwrap();

                        let line_label = Label::builder().label(format!("{}", li)).halign(Align::End).build();

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

    let button_container = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .margin_end(10)
        .margin_top(10)
        .margin_start(10)
        .margin_bottom(10)
        .build();

    let open_file_button = Button::builder()
        .label("Open")
        .hexpand(true)
        .build();

    button_container.append(&simulate_button);
    button_container.append(&open_file_button);

    container_box.append(&button_container);
    container_box.append(&file_display);
    container_box.append(&separator_top);
    container_box.append(&scrolled_window);
    container_box.append(&separator_bottom);
    container_box.append(&stats_showcase);

    window.set_child(Some(&container_box));

    open_file_button.connect_clicked(clone!(@weak window, @weak file_display_spinner, @weak simulate_button => 
        move |_| {
            let file_dialogue = FileDialog::new();
            simulate_button.set_sensitive(false);
            file_display_spinner.set_spinning(true);
            file_dialogue.open(Window::NONE, Cancellable::NONE, move |result| {
                match result {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            window.set_path_buf(path);
                        }
                        simulate_button.set_sensitive(true);
                    },
                    Err(err) => {
                        if let Some(dialog_error) = err.kind::<DialogError>() {
                            dbg!(dialog_error);
                        }
                    }
                }
                file_display_spinner.set_spinning(false);
            })
        }
    ));


    window.present();
    0
}

type CacheLineVec = Vec<Vec<CacheEntry>>;

fn run_sim(path: &PathBuf) -> Result<SimResult, Box<dyn Error>> {
    let (cache, addrs) = sim::read(path)?;

    let (lines, stats) = sim::simulate(&cache, &addrs);

    Ok((lines, cache, addrs, stats))
}
