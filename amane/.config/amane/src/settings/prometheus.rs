/*
 * prometheus, the desktop with the AOC monitor (docs/machine.md): one amd gpu,
 * so its render node is named rather than looked for; everything else is the
 * default
 */

use super::{RenderNode, Settings};

pub fn settings() -> Settings {
    Settings {
        render_node: RenderNode::Path("/dev/dri/renderD128"),
        ..Settings::default()
    }
}
