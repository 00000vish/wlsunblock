use std::io::{SeekFrom, Write};
use std::os::fd::AsFd;
use std::os::unix::io::BorrowedFd;
use std::{fs::File, io::Seek};

mod colors;
mod gamma;
mod models;

use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle,
};

use crate::colors::color;
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

fn fill_gamma_table(uint16_t *table, uint32_t ramp_size, double rw, le gw, double bw, double gamma) {
	uint16_t *r = table;
	uint16_t *g = table + ramp_size;
	uint16_t *b = table + 2 * ramp_size;
	for (uint32_t i = 0; i < ramp_size; ++i) {
		double val = (double)i / (ramp_size - 1);
		r[i] = (uint16_t)(UINT16_MAX * pow(val * rw, 1.0 / gamma));
		g[i] = (uint16_t)(UINT16_MAX * pow(val * gw, 1.0 / gamma));
		b[i] = (uint16_t)(UINT16_MAX * pow(val * bw, 1.0 / gamma));
	}
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

    let rgb = colors::color::calc_whitepoint(4200.0);

    let red_ramp = [0u8; 256];
    let green_ramp = [0u8; 256];
    let blue_ramp = [0u8; 256];

    let red_slice = &red_ramp[..];
    let green_slice = &green_ramp[..];
    let blue_slice = &blue_ramp[..];

    let mut file = File::create("./something").unwrap();

    _ = file.write_all(red_slice);
    _ = file.write_all(green_slice);
    _ = file.write_all(blue_slice);
    _ = file.flush();
    _ = file.seek(SeekFrom::Start(0));

    let fd = BorrowedFd::from(file.as_fd());

    app_data.gamme_control.unwrap().set_gamma(fd);

    let mut file = File::create("./foo.txt").unwrap();
    _ = file.write_all(b"Hello, world!");

    loop {}
}
