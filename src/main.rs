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

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut app_data = AppData {
        output: None,
        gamme_control: None,
    };

    event_queue.roundtrip(&mut app_data).unwrap();

    let _rgb = colors::color::calc_whitepoint(4200.0);

    let mut file = File::create("./temp").unwrap();

    _ = file.write_all(&mut format!("{}", _rgb.r).into_bytes());
    _ = file.write_all(&mut format!("{}", _rgb.g).into_bytes());
    _ = file.write_all(&mut format!("{}", _rgb.b).into_bytes());
    _ = file.flush();
    _ = file.seek(SeekFrom::Start(0));

    let fd = BorrowedFd::from(file.as_fd());

    app_data.gamme_control.unwrap().set_gamma(fd);

    loop {}
}
