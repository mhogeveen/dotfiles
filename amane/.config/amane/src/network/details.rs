/*
 * the interface, address and gateway of the connection that carries the default
 * route; amane's Network doesn't read these, so they come straight from
 * NetworkManager here, the same way amane asks it everything else
 *
 * every call waits on d-bus, so only from a thread, never from a view
 */

use amane::{Argument, Bus, Value};

const NAME: &str = "org.freedesktop.NetworkManager";
const PATH: &str = "/org/freedesktop/NetworkManager";
const ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const IP4: &str = "org.freedesktop.NetworkManager.IP4Config";
const DEVICE: &str = "org.freedesktop.NetworkManager.Device";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Details {
    // like wlp2s0
    pub interface: String,

    // like 192.168.1.23/24, empty before an address is handed out
    pub address: String,

    // empty on a network without one
    pub gateway: String,
}

// None while offline
pub fn fetch() -> Option<Details> {
    let primary = property(PATH, NAME, "PrimaryConnection");
    let primary = primary.text();

    // networkmanager says "/" when there is no primary connection
    if primary.is_empty() || primary == "/" {
        return None;
    }

    let active = properties(primary, ACTIVE);

    let interface = active
        .get("Devices")
        .list()
        .first()
        .map(|device| property(device.text(), DEVICE, "Interface"))
        .map(|interface| String::from(interface.text()))
        .unwrap_or_default();

    let config = active.get("Ip4Config").text();

    let (address, gateway) = if config.is_empty() || config == "/" {
        (String::new(), String::new())
    } else {
        let ip4 = properties(config, IP4);

        let address = ip4
            .get("AddressData")
            .list()
            .first()
            .map(|data| {
                format!(
                    "{}/{}",
                    data.get("address").text(),
                    data.get("prefix").number()
                )
            })
            .unwrap_or_default();

        (address, String::from(ip4.get("Gateway").text()))
    };

    Some(Details {
        interface,
        address,
        gateway,
    })
}

fn property(path: &str, interface: &str, name: &str) -> Value {
    Bus::system().property(NAME, path, interface, name)
}

fn properties(path: &str, interface: &str) -> Value {
    Bus::system().call(
        NAME,
        path,
        "org.freedesktop.DBus.Properties",
        "GetAll",
        &[Argument::from(interface)],
    )
}
