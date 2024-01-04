use std::fs::File;
use std::os::fd::AsFd;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::BorrowedFd;

mod gamma;

use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle,
};

use crate::gamma::gamma_protocol::zwlr_gamma_control_v1;

struct AppData {
    output: Option<wl_output::WlOutput>,
    gamme_control: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        data: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} (v{})", name, interface, version);
            if interface == "wl_output" {
                let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
                data.output = Some(output);
                println!("got it ");
            }
            if interface == "zwlr_gamma_control_manager_v1" {
                let output = registry.bind::<zwlr_gamma_control_v1::ZwlrGammaControlV1, _, _>(
                    name,
                    version,
                    qh,
                    (),
                );
                data.gamme_control = Some(output);
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        print!("got it");
    }
}

impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        _: zwlr_gamma_control_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        print!("got it");
    }
}

fn calculate_gamma(g: f64, index: u8) -> u8 {
    (255.0 * ((index as f64 / 255.0).powf(1.0 / g)) + 0.5).min(255.0) as u8
}

fn main() {
    // Create a Wayland connection by connecting to the server through the
    // environment-provided configuration.
    let conn = Connection::connect_to_env().unwrap();

    // Retrieve the WlDisplay Wayland object from the connection. This object is
    // the starting point of any Wayland program, from which all other objects will
    // be created.
    let display = conn.display();

    // Create an event queue for our event processing
    let mut event_queue = conn.new_event_queue();
    // And get its handle to associated new objects to it
    let qh = event_queue.handle();

    // Create a wl_registry object by sending the wl_display.get_registry request
    // This method takes two arguments: a handle to the queue the newly created
    // wl_registry will be assigned to, and the user-data that should be associated
    // with this registry (here it is () as we don't need user-data).
    let _registry = display.get_registry(&qh, ());

    // At this point everything is ready, and we just need to wait to receive the events
    // from the wl_registry, our callback will print the advertized globals.
    println!("Advertized globals:");

    // To actually receive the events, we invoke the `sync_roundtrip` method. This method
    // is special and you will generally only invoke it during the setup of your program:
    // it will block until the server has received and processed all the messages you've
    // sent up to now.
    //
    // In our case, that means it'll block until the server has received our
    // wl_display.get_registry request, and as a reaction has sent us a batch of
    // wl_registry.global events.
    //
    // `sync_roundtrip` will then empty the internal buffer of the queue it has been invoked
    // on, and thus invoke our `Dispatch` implementation that prints the list of advertized
    // globals.
    let mut app_data = AppData {
        output: None,
        gamme_control: None,
    };

    event_queue.roundtrip(&mut app_data).unwrap();

    let file = File::open("path/to/file").unwrap();
    let fd = BorrowedFd::from(file.as_fd());

    let red_gamma = 0.5;
    let green_gamma = 0.5;
    let blue_gamma = 0.5;

    let mut red_ramp = [0u8; 256];
    let mut green_ramp = [0u8; 256];
    let mut blue_ramp = [0u8; 256];

    for i in 0..256 {
        red_ramp[i] = calculate_gamma(red_gamma, i as u8);
        green_ramp[i] = calculate_gamma(green_gamma, i as u8);
        blue_ramp[i] = calculate_gamma(blue_gamma, i as u8);
    }

    let mut file = unsafe { std::fs::File::from_raw_fd(fd.into_raw_fd()) };

    let red_slice = &red_ramp[..];
    let green_slice = &green_ramp[..];
    let blue_slice = &blue_ramp[..];

    file.write_all(red_slice)?;
    file.write_all(green_slice)?;
    file.write_all(blue_slice)?;

    app_data.gamme_control.unwrap().set_gamma(fd);

    loop {}
}
