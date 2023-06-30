use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, error::Error, thread};

use gtk::gio::{ApplicationFlags, ApplicationCommandLine};
use gtk::glib::{MainContext, Priority};
use gtk::{prelude::*, ApplicationWindow, ScrolledWindow, PolicyType, Button, Orientation};
use gtk::{Application, glib};
use sim::{CacheEntry, CacheStats};
use glib::clone;
use glib::prelude::*;

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
    Success((CacheLineVec, CacheStats)),
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
        .hscrollbar_policy(PolicyType::Never)
        .min_content_width(260)
        .build();

    let simulate_button = Button::builder()
        .label("Simulate")
        .build();

    let (sender, receiver) = MainContext::channel(Priority::default());

    simulate_button.connect_clicked(move |_| {
        let sender = sender.clone();
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

    receiver.attach(None, clone!(@weak simulate_button, @weak scrolled_window => @default-return Continue(false),
        move |result| {
            println!("handling");
            match result {
                SimulationCommunication::Success((lines, _)) => {
                    simulate_button.set_sensitive(true);
                    for line in lines {
                        for cache_entry in line {
                            print!("{}", cache_entry);
                        }
                        println!();
                    }
                },
                SimulationCommunication::Failure => {
                    simulate_button.set_sensitive(true);
                },
                SimulationCommunication::Run => {
                    simulate_button.set_sensitive(false);
                }
            }
            println!("handled");
            Continue(true)
        }
    ));

    let container_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    container_box.append(&simulate_button);
    container_box.append(&scrolled_window);

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

fn run_sim(path: &PathBuf) -> Result<(CacheLineVec, sim::CacheStats), Box<dyn Error>> {
    let (cache, addrs) = sim::read(path)?;

    Ok(sim::simulate(&cache, &addrs))
}
