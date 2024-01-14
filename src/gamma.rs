use wayland_client;
// import objects from the core protocol if needed
use wayland_client::protocol::*;

// This module hosts a low-level representation of the protocol objects
// you will not need to interact with it yourself, but the code generated
// by the generate_client_code! macro will use it
pub mod __interfaces {
    // import the interfaces from the core protocol if needed
    use wayland_client::protocol::__interfaces::*;
    wayland_scanner::generate_interfaces!("src/protocols/wlr-gamma-control-unstable-v1.xml");
}
use self::__interfaces::*;

wayland_scanner::generate_client_code!("src/protocols/wlr-gamma-control-unstable-v1.xml");
