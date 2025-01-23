use egui::Vec2;

use crate::app::components::File;

impl super::Plotter {
    pub(super) fn manipulate_file(
        &mut self,
        active_file: &mut File,
        modifiers: [bool; 3],
        axes_spans: (f64, f64),
        drag: Vec2,
    ) {
        // How much did the mouse move?
        let Vec2 { x: dx, y: dy } = drag;
        match modifiers {
            // Alt key is pressed → change xoffset.
            [true, false, false] => {
                active_file.properties.xoffset += axes_spans.0 * 0.001 * (dx as f64);
            }
            // Ctrl key is pressed → change yoffset.
            [false, true, false] => {
                active_file.properties.yoffset -= axes_spans.1 * 0.001 * (dy as f64);
            }
            // Shift is pressed → change yscale.
            [false, false, true] => {
                let yscale = active_file.properties.yscale;
                active_file.properties.yscale -= yscale * 0.01 * (dy as f64);
            }
            // If several modifiers are pressed at the same time,
            // we ignore the input.
            _ => (),
        }
    }
}
