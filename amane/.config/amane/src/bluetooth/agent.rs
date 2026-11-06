/*
 * the pairing agent: BlueZ asks it whether a pairing code matches, and the
 * bluetooth panel shows the code with Confirm and Cancel
 *
 * BlueZ calls this object's org.bluez.Agent1 methods and waits for an answer;
 * a confirmation is answered once you press Confirm. amane's d-bus can't send
 * an error answer, so Cancel asks BlueZ to cancel the pairing instead, which
 * ends the waiting call from BlueZ's side
 *
 * devices that need a code typed in on this computer aren't supported, those
 * are cancelled with a hint to pair them with bluetoothctl
 */

use std::sync::Mutex;
use std::thread;

use amane::{Argument, Bus, Method, Service, Urgency};

use crate::notifications;
use crate::panels::{BLUETOOTH, Panels};

const BLUEZ: &str = "org.bluez";
const AGENT: &str = "org.bluez.Agent1";
const AGENT_MANAGER: &str = "org.bluez.AgentManager1";
const DEVICE: &str = "org.bluez.Device1";
const PATH: &str = "/org/amane/bluetooth_agent";

// answers with a yes or no, and can show a code: what a panel with buttons can do
const CAPABILITY: &str = "DisplayYesNo";

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    // does this code match the one on the device? Confirm or Cancel
    Confirm(String),

    // pair with this device? Confirm or Cancel; for devices pairing with this computer
    Authorize,

    // type this code on the device, nothing to answer
    Display(String),

    // the device wants a code typed in here, which isn't supported
    NeedsCode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub device: String,
    pub name: String,
    pub kind: Kind,
}

impl Request {
    // Confirm and Cancel only mean something while BlueZ waits for an answer
    pub fn answerable(&self) -> bool {
        matches!(self.kind, Kind::Confirm(_) | Kind::Authorize)
    }
}

// what the panel and bar show; the waiting call itself is kept in WAITING
pub struct Agent {
    request: Option<Request>,
}

impl Service for Agent {
    fn new() -> Self {
        Self { request: None }
    }

    // changes only when BlueZ calls, nothing to poll
    fn listen() {}
}

impl Agent {
    pub fn request(&self) -> Option<&Request> {
        self.request.as_ref()
    }
}

/*
 * BlueZ's call that waits for Confirm or Cancel; not in the service, since
 * nothing draws it and answering it shouldn't redraw anything by itself
 */
static WAITING: Mutex<Option<Method>> = Mutex::new(None);

fn waiting() -> std::sync::MutexGuard<'static, Option<Method>> {
    WAITING.lock().unwrap_or_else(|error| error.into_inner())
}

// listens for BlueZ's calls on its own thread, for as long as the shell runs
pub fn start() {
    thread::spawn(|| {
        // watched before registering, so the first call isn't missed
        let methods = Bus::system().methods(PATH, AGENT);

        register();

        for method in methods {
            answer(method);
        }
    });
}

/*
 * tells BlueZ to ask this agent, also for pairings other programs start; again
 * after bluetoothd restarted, which forgets every agent. waits on d-bus
 */
pub fn register() {
    let path = || Argument::Path(String::from(PATH));

    Bus::system().call(
        BLUEZ,
        "/org/bluez",
        AGENT_MANAGER,
        "RegisterAgent",
        &[path(), Argument::from(CAPABILITY)],
    );

    Bus::system().call(
        BLUEZ,
        "/org/bluez",
        AGENT_MANAGER,
        "RequestDefaultAgent",
        &[path()],
    );
}

fn answer(method: Method) {
    let arguments = method.arguments();

    let device = arguments
        .first()
        .map(|device| String::from(device.text()))
        .unwrap_or_default();

    let name = String::from(method.name());

    match name.as_str() {
        "RequestConfirmation" => {
            let passkey = arguments.get(1).map(|key| key.number() as u32).unwrap_or(0);

            wait(method, &device, Kind::Confirm(format!("{passkey:06}")));
        }

        "RequestAuthorization" => wait(method, &device, Kind::Authorize),

        "DisplayPasskey" => {
            let passkey = arguments.get(1).map(|key| key.number() as u32).unwrap_or(0);

            show(&device, Kind::Display(format!("{passkey:06}")));

            method.reply(&[]);
        }

        "DisplayPinCode" => {
            let code = arguments.get(1).map(|code| String::from(code.text()));

            show(&device, Kind::Display(code.unwrap_or_default()));

            method.reply(&[]);
        }

        // only devices that are paired already ask for this
        "AuthorizeService" => method.reply(&[]),

        // typing a code in here isn't supported; left unanswered, cancelling ends it
        "RequestPasskey" | "RequestPinCode" => {
            show(&device, Kind::NeedsCode);

            cancel_pairing(device);
        }

        // BlueZ gave up waiting, or let this agent go
        "Cancel" | "Release" => {
            clear();

            method.reply(&[]);
        }

        _ => method.reply(&[]),
    }
}

// keeps BlueZ's call until Confirm or Cancel
fn wait(method: Method, device: &str, kind: Kind) {
    // a newer request replaces one nobody answered, BlueZ cancels that one itself
    *waiting() = Some(method);

    show(device, kind);

    // a device pairing by itself while the panel is closed would go unnoticed
    if !Panels::read().is_open(BLUETOOTH) {
        let name = device_name(device);

        thread::spawn(move || {
            notifications::send(
                "Bluetooth",
                "Pairing request",
                &format!("{name} wants to pair, open Bluetooth to confirm"),
                Urgency::Normal,
                None,
            );
        });
    }
}

fn show(device: &str, kind: Kind) {
    let request = Request {
        device: String::from(device),
        name: device_name(device),
        kind,
    };

    Agent::write().request = Some(request);
}

// clears what the panel shows, and lets go of a call still waiting
pub fn clear() {
    waiting().take();

    let mut agent = Agent::write();

    if agent.request.is_some() {
        agent.request = None;
    }
}

// the code matches: answering the waiting call lets the pairing go on
pub fn confirm() {
    if let Some(method) = waiting().take() {
        method.reply(&[]);
    }

    let mut agent = Agent::write();

    // a code to type on the device stays up until the pairing ends
    if agent.request.as_ref().is_some_and(Request::answerable) {
        agent.request = None;
    }
}

// the code doesn't match, or you changed your mind
pub fn cancel() {
    let device = Agent::read()
        .request()
        .map(|request| request.device.clone());

    waiting().take();

    Agent::write().request = None;

    if let Some(device) = device {
        cancel_pairing(device);
    }
}

// a code shown for typing on the device goes away once the pairing is over
pub fn pairing_done() {
    let mut agent = Agent::write();

    if agent
        .request
        .as_ref()
        .is_some_and(|request| matches!(request.kind, Kind::Display(_)))
    {
        agent.request = None;
    }
}

fn cancel_pairing(device: String) {
    thread::spawn(move || {
        Bus::system().call(BLUEZ, &device, DEVICE, "CancelPairing", &[]);
    });
}

// waits on d-bus, but BlueZ answers property reads right away
fn device_name(device: &str) -> String {
    let alias = Bus::system().property(BLUEZ, device, DEVICE, "Alias");

    match alias.text() {
        "" => String::from("A device"),
        name => String::from(name),
    }
}
