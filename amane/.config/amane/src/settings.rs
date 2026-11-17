/*
 * the values that differ between machines (programs, systemd units, the gpu,
 * sensor drivers, folders, the weather place), compiled in: Settings::default()
 * is every host's, and a host with a file in src/settings/ changes only what it
 * needs (`..Settings::default()`). the host is picked at start by
 * /proc/sys/kernel/hostname; one without a file of its own gets the defaults
 *
 * compiled rather than read from a file, so a typo or a wrong type fails
 * `amane compile` instead of showing up after a restart. what can only fail at
 * run time (a program that isn't installed, a missing folder) fails where it's
 * used, as it did before
 *
 * adding a host: a file in src/settings/ with a `settings()` like
 * prometheus.rs, a `mod` line and a line in for_host()
 */

mod prometheus;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::weather::Place;

const HOSTNAME: &str = "/proc/sys/kernel/hostname";

// where RenderNode::Auto looks, taking the lowest renderD number
const RENDER_NODES: &str = "/dev/dri";
const RENDER_NODE_PREFIX: &str = "renderD";

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    // the launcher's >>command runs in this; its words come before `sh -c ...`, so `alacritty -e` works
    pub terminal: &'static str,

    // the screenshot region is fed to this on stdin to annotate (satty's flags)
    pub annotator: &'static str,

    // screen recordings (wf-recorder's flags)
    pub recorder: &'static str,

    // the user units the system panel's switches start and stop
    pub idle_unit: &'static str,
    pub night_light_unit: &'static str,

    // the gpu recordings encode on through vaapi
    pub render_node: RenderNode,

    // hwmon driver names; the first sensor found for each is shown
    pub cpu_drivers: &'static [&'static str],
    pub gpu_drivers: &'static [&'static str],

    /*
     * satty saves annotated screenshots to its own output-filename
     * (~/.nixos/home/satty.nix), which should be in this folder too
     */
    pub wallpapers: PathBuf,
    pub screenshots: PathBuf,
    pub screencasts: PathBuf,

    // the place until one is set with `amane ipc call weather location`
    pub weather: Place,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderNode {
    // the first /dev/dri/renderD* there is, software when there's none
    Auto,
    Path(&'static str),
    // always encode in software, for a gpu without vaapi; no host needs it yet
    #[allow(dead_code)]
    None,
}

impl RenderNode {
    // the node to encode on, None to encode in software
    pub fn path(&self) -> Option<PathBuf> {
        match self {
            RenderNode::Auto => first_render_node(&render_node_names()),
            RenderNode::Path(path) => Some(PathBuf::from(path)),
            RenderNode::None => None,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            terminal: "foot",
            annotator: "satty",
            recorder: "wf-recorder",
            idle_unit: "hypridle.service",
            night_light_unit: "wlsunset.service",
            render_node: RenderNode::Auto,
            cpu_drivers: &["k10temp", "zenpower", "coretemp"],
            gpu_drivers: &["amdgpu", "nouveau", "radeon", "i915", "xe"],
            wallpapers: in_home("Pictures/Wallpapers"),
            screenshots: in_home("Pictures/Screenshots"),
            screencasts: in_home("Videos/Screencasts"),
            weather: Place {
                latitude: 52.223,
                longitude: 5.176,
                name: String::from("Hilversum"),
            },
        }
    }
}

// this host's settings, by its name
fn for_host(host: &str) -> Settings {
    match host {
        "prometheus" => prometheus::settings(),
        _ => Settings::default(),
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

/*
 * picked the first time anything asks; a plain static set once, so views can
 * read it without the lock rules services have
 */
pub fn settings() -> &'static Settings {
    SETTINGS.get_or_init(|| {
        let host = fs::read_to_string(HOSTNAME).unwrap_or_default();

        for_host(host.trim())
    })
}

// a folder under the home folder, like in_home("Pictures/Wallpapers")
pub fn in_home(relative: &str) -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(relative)
}

/*
 * where the config folders are: $XDG_CONFIG_HOME or ~/.config, absolute only;
 * mango's config is found from here
 */
pub fn config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| in_home(".config"))
}

// the names in /dev/dri, unsorted
fn render_node_names() -> Vec<String> {
    fs::read_dir(RENDER_NODES)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

// renderD128 before renderD129, by number; None when there's no gpu
fn first_render_node(names: &[String]) -> Option<PathBuf> {
    names
        .iter()
        .filter_map(|name| {
            let number: u32 = name.strip_prefix(RENDER_NODE_PREFIX)?.parse().ok()?;

            Some((number, name))
        })
        .min()
        .map(|(_, name)| Path::new(RENDER_NODES).join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_host_gets_the_defaults() {
        assert_eq!(for_host("somewhere-else"), Settings::default());
        assert_eq!(for_host(""), Settings::default());
    }

    #[test]
    fn prometheus_changes_only_the_render_node() {
        let prometheus = for_host("prometheus");

        assert_eq!(
            prometheus.render_node,
            RenderNode::Path("/dev/dri/renderD128")
        );
        assert_eq!(
            Settings {
                render_node: RenderNode::Auto,
                ..prometheus
            },
            Settings::default()
        );
    }

    #[test]
    fn auto_takes_the_lowest_render_node() {
        let names = [
            String::from("card1"),
            String::from("renderD129"),
            String::from("renderD128"),
        ];

        assert_eq!(
            first_render_node(&names),
            Some(PathBuf::from("/dev/dri/renderD128"))
        );
        assert_eq!(first_render_node(&[String::from("card0")]), None);
        assert_eq!(first_render_node(&[]), None);
    }

    #[test]
    fn a_path_or_none_is_used_as_written() {
        assert_eq!(
            RenderNode::Path("/dev/dri/renderD129").path(),
            Some(PathBuf::from("/dev/dri/renderD129"))
        );
        assert_eq!(RenderNode::None.path(), None);
    }
}
