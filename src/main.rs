use std::{env, error::Error, thread};

use gtk::glib::{MainContext, Priority};
use gtk::{prelude::*, ApplicationWindow, ScrolledWindow, PolicyType, Button, Orientation};
use gtk::{Application, glib};
use sim::CacheEntry;
use glib::clone;

mod sim;

const APP_ID: &str = "com.github.maxi0604.CacheCache";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
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
        thread::spawn(move || {
            sender.send(None).expect("Could not send through channel");
            
            match run_sim() {
                Ok(result) => {
                    sender.send(Some(result)).expect("Could not send through channel");
                },
                Err(_) => {
                    // TODO: Fehlerbehandlung
                }
            }

        });
    });

    receiver.attach(None, clone!(@weak simulate_button, @weak scrolled_window => @default-return Continue(false),
        move |result| {
            match result {
                Some(_) => {
                    simulate_button.set_sensitive(false);
                },
                None => {
                    simulate_button.set_sensitive(true);
                }
            }
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

    window.present()
}

type CacheLineVec = Vec<Vec<CacheEntry>>;

fn run_sim() -> Result<(CacheLineVec, sim::CacheStats), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(Box::from(sim::InvalidArgumentsError));
    }

    let (cache, addrs) = sim::read(&args[1])?;

    Ok(sim::simulate(&cache, &addrs))
}
