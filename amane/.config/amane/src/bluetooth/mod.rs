/*
 * what the bluetooth panel adds on top of amane's Bluetooth service: scanning
 * only while the panel is open, power that gets past rfkill, and connect, pair
 * and forget that say whether they worked
 *
 * amane runs those calls without saying how they went, so they're made here
 * on a thread and the device is checked afterwards
 *
 * lock order: the tick copies what it needs out of Bluetooth and Panels and
 * lets go before writing BluetoothPanel, and nothing holds BluetoothPanel's
 * write lock while reading another service
 */

pub mod agent;

use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::{Argument, Bluetooth, BluetoothDevice, Bus, Service, Value};

use crate::audio::{ICON_HEADPHONES, ICON_SPEAKER};
use crate::panels::{BLUETOOTH, Panels};

const BLUEZ: &str = "org.bluez";
const ADAPTER: &str = "org.bluez.Adapter1";
const DEVICE: &str = "org.bluez.Device1";

// how long rfkill takes to let go before the adapter can power up
const UNBLOCK_SETTLE: Duration = Duration::from_millis(300);

// nerd font material design glyphs
pub const ICON_ON: &str = "\u{f00af}";
pub const ICON_CONNECTED: &str = "\u{f00b1}";
pub const ICON_OFF: &str = "\u{f00b2}";

// a glyph for the kind of device, from the freedesktop icon name BlueZ gives it
pub fn device_icon(device: &BluetoothDevice) -> &'static str {
    let icon = device.icon();

    if icon.starts_with("audio-head") {
        ICON_HEADPHONES
    } else if icon.starts_with("audio") {
        ICON_SPEAKER
    } else if icon == "input-mouse" || icon == "input-tablet" {
        "\u{f037d}"
    } else if icon == "input-keyboard" {
        "\u{f030c}"
    } else if icon == "input-gaming" {
        "\u{f0296}"
    } else if icon.starts_with("phone") {
        "\u{f011c}"
    } else if icon.starts_with("computer") {
        "\u{f0322}"
    } else {
        ICON_ON
    }
}

/*
 * a device that never said its name shows its address instead; nearby strangers
 * mostly, so they're left out until paired
 */
pub fn unnamed(device: &BluetoothDevice) -> bool {
    device
        .name()
        .replace('-', ":")
        .eq_ignore_ascii_case(device.address())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Job {
    PoweringOn,
    PoweringOff,
    Connecting,
    Disconnecting,
    Pairing,
    Forgetting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Busy {
    // the device's path, None for the adapter itself
    pub device: Option<String>,
    pub job: Job,
}

pub struct BluetoothPanel {
    busy: Option<Busy>,

    // what went wrong last, like "Couldn't connect to WH-1000XM4"
    failure: Option<String>,
}

impl Service for BluetoothPanel {
    fn new() -> Self {
        Self {
            busy: None,
            failure: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    // what the last tick saw is kept here, not in the service, so changes to it redraw nothing
    fn listen() {
        let mut last = Previous::default();

        loop {
            thread::sleep(Self::interval());

            last = tick(&last);
        }
    }
}

/*
 * scans while the panel is open and the adapter is on, and registers the
 * pairing agent again when bluetoothd comes back, since it forgets agents
 */
// what one tick saw, for the next to compare with
#[derive(Default)]
struct Previous {
    open: bool,
    available: bool,
    powered: bool,
}

fn tick(last: &Previous) -> Previous {
    let open = Panels::read().is_open(BLUETOOTH);

    let (available, powered, scanning) = {
        let bluetooth = Bluetooth::read();

        (
            bluetooth.available(),
            bluetooth.powered(),
            bluetooth.scanning(),
        )
    };

    let (was_open, was_available, was_powered) = (last.open, last.available, last.powered);

    if available && !was_available {
        agent::register();
    }

    /*
     * scanning slows wi-fi and uses power, so only while you're looking; asked
     * once when the panel opens or the adapter comes on, not every tick, since
     * amane only notices the scan up to 2 seconds later
     */
    let became_ready = open && powered && (!was_open || !was_powered);

    if became_ready && !scanning {
        Bluetooth::start_scan();
    } else if !open && was_open && scanning {
        Bluetooth::stop_scan();
    }

    Previous {
        open,
        available,
        powered,
    }
}

/*
 * starts the tick at startup, not when the panel first opens: it registers the
 * pairing agent again whenever bluetoothd comes back, panel open or not
 */
pub fn start() {
    drop(BluetoothPanel::read());
}

// reading, safe inside views
impl BluetoothPanel {
    // what this device is waiting on, if anything
    pub fn job_for(&self, device: &str) -> Option<Job> {
        self.busy
            .as_ref()
            .filter(|busy| busy.device.as_deref() == Some(device))
            .map(|busy| busy.job)
    }

    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

/*
 * writing, from clicks; one thing at a time, a click while something runs is
 * ignored. the work happens on a thread of its own and never waits here
 */
impl BluetoothPanel {
    pub fn power(on: bool) {
        let job = if on {
            Job::PoweringOn
        } else {
            Job::PoweringOff
        };

        run(None, job, move || set_powered(on));
    }

    pub fn connect(device: &BluetoothDevice) {
        let name = String::from(device.name());
        let path = String::from(device.path());

        run(Some(device.path()), Job::Connecting, move || {
            call(&path, "Connect");

            if device_bool(&path, "Connected") {
                Ok(())
            } else {
                Err(format!("Couldn't connect to {name}"))
            }
        });
    }

    pub fn disconnect(device: &BluetoothDevice) {
        let name = String::from(device.name());
        let path = String::from(device.path());

        run(Some(device.path()), Job::Disconnecting, move || {
            call(&path, "Disconnect");

            if device_bool(&path, "Connected") {
                Err(format!("Couldn't disconnect {name}"))
            } else {
                Ok(())
            }
        });
    }

    /*
     * pairs, trusts so it may reconnect by itself, then connects, like other
     * bluetooth menus do; a device with a code asks the agent along the way
     */
    pub fn pair(device: &BluetoothDevice) {
        let name = String::from(device.name());
        let path = String::from(device.path());

        run(Some(device.path()), Job::Pairing, move || {
            call(&path, "Pair");

            agent::pairing_done();

            if !device_bool(&path, "Paired") {
                return Err(format!("Couldn't pair with {name}"));
            }

            Bus::system().set_property(BLUEZ, &path, DEVICE, "Trusted", Argument::from(true));

            call(&path, "Connect");

            if device_bool(&path, "Connected") {
                Ok(())
            } else {
                Err(format!("Paired with {name}, but couldn't connect"))
            }
        });
    }

    pub fn forget(device: &BluetoothDevice) {
        let name = String::from(device.name());
        let path = String::from(device.path());

        run(Some(device.path()), Job::Forgetting, move || {
            let Some(adapter) = adapter() else {
                return Err(String::from("No Bluetooth adapter"));
            };

            Bus::system().call(
                BLUEZ,
                &adapter,
                ADAPTER,
                "RemoveDevice",
                &[Argument::Path(path.clone())],
            );

            // a removed device has no properties left to read
            let gone = Bus::system().property(BLUEZ, &path, DEVICE, "Address") == Value::Nothing;

            if gone {
                Ok(())
            } else {
                Err(format!("Couldn't forget {name}"))
            }
        });
    }

    pub fn dismiss_failure() {
        BluetoothPanel::write().failure = None;
    }
}

fn run(device: Option<&str>, job: Job, work: impl FnOnce() -> Result<(), String> + Send + 'static) {
    {
        let mut panel = BluetoothPanel::write();

        if panel.busy.is_some() {
            return;
        }

        panel.busy = Some(Busy {
            device: device.map(String::from),
            job,
        });

        panel.failure = None;
    }

    thread::spawn(move || {
        let result = work();

        // so the list shows the change now, not at amane's next poll
        Bluetooth::write().update();

        let mut panel = BluetoothPanel::write();

        panel.busy = None;
        panel.failure = result.err();
    });
}

/*
 * on: rfkill's soft block stops BlueZ from powering the adapter, and your
 * user may lift it; the panel then starts scanning on its next tick
 */
fn set_powered(on: bool) -> Result<(), String> {
    if on {
        let _ = Command::new("rfkill")
            .args(["unblock", "bluetooth"])
            .status();

        thread::sleep(UNBLOCK_SETTLE);
    }

    let Some(adapter) = adapter() else {
        return Err(String::from("No Bluetooth adapter"));
    };

    Bus::system().set_property(BLUEZ, &adapter, ADAPTER, "Powered", Argument::from(on));

    let powered = Bus::system()
        .property(BLUEZ, &adapter, ADAPTER, "Powered")
        .bool();

    match (on, powered) {
        (true, false) => Err(String::from("Couldn't turn on Bluetooth")),
        (false, true) => Err(String::from("Couldn't turn off Bluetooth")),
        _ => Ok(()),
    }
}

// connecting or pairing can take seconds, the caller is always a thread
fn call(device: &str, method: &str) {
    Bus::system().call(BLUEZ, device, DEVICE, method, &[]);
}

fn device_bool(device: &str, property: &str) -> bool {
    Bus::system()
        .property(BLUEZ, device, DEVICE, property)
        .bool()
}

// the first adapter, like amane's Bluetooth uses
fn adapter() -> Option<String> {
    let objects = Bus::system().call(
        BLUEZ,
        "/",
        "org.freedesktop.DBus.ObjectManager",
        "GetManagedObjects",
        &[],
    );

    let Value::Map(objects) = objects else {
        return None;
    };

    objects
        .iter()
        .find(|(_, interfaces)| interfaces.get(ADAPTER) != &Value::Nothing)
        .map(|(path, _)| path.clone())
}
