/*
 * saved wi-fi profiles: amane's Network can join a saved network but not forget
 * one, which is what fixes a profile saved with a mistyped password
 *
 * every call waits on d-bus, so only from a thread, never from a view
 */

use amane::{Bus, Value};

const NAME: &str = "org.freedesktop.NetworkManager";
const SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";
const SETTINGS: &str = "org.freedesktop.NetworkManager.Settings";
const CONNECTION: &str = "org.freedesktop.NetworkManager.Settings.Connection";

/*
 * deletes every saved profile for this network; Bus::call answers the same for
 * a refused call as for one that returns nothing, so the list is read again to
 * see whether they're really gone
 */
pub fn forget(ssid: &str) -> Result<(), String> {
    let paths = find(ssid);

    if paths.is_empty() {
        return Err(format!("{ssid} isn't saved"));
    }

    for path in &paths {
        Bus::system().call(NAME, path, CONNECTION, "Delete", &[]);
    }

    if find(ssid).is_empty() {
        Ok(())
    } else {
        Err(String::from("needs NetworkManager permission"))
    }
}

// the paths of the saved profiles for this network
fn find(ssid: &str) -> Vec<String> {
    let profiles = Bus::system().call(NAME, SETTINGS_PATH, SETTINGS, "ListConnections", &[]);

    profiles
        .list()
        .iter()
        .map(|profile| profile.text())
        .filter(|path| {
            let settings = Bus::system().call(NAME, path, CONNECTION, "GetSettings", &[]);

            // wired and vpn profiles have no wifi group, so no name to match
            name(settings.get("802-11-wireless").get("ssid")) == ssid
        })
        .map(String::from)
        .collect()
}

// the network name comes as raw bytes, and is almost always utf-8
fn name(bytes: &Value) -> String {
    let raw: Vec<u8> = bytes
        .list()
        .iter()
        .map(|byte| byte.number() as u8)
        .collect();

    String::from_utf8_lossy(&raw).into_owned()
}
