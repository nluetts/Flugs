use crate::app::FileHandler;

impl super::Plotter {
    pub(super) fn manipulate_plots(
        &mut self,
        hovered_item_id: egui::Id,
        file_handler: &mut FileHandler,
        modifiers: [bool; 3],
        ctx: &egui::Context,
    ) {
        log::debug!(
            "manipulating file: {:?}, {:?}",
            &self.files_plot_ids,
            &hovered_item_id
        );

        for (_, fid) in self
            .files_plot_ids
            .drain()
            .filter(|(egui_id, _)| *egui_id == hovered_item_id)
        {
            // We can unwrap here, since self.files_plot_ids can only contain
            // valid file IDs
            let file = file_handler
                .registry
                .get_mut(&fid)
                .expect("A file ID handed over from the Plotter was invalid.");

            // How much did the mouse move?
            let egui::Vec2 { x: dx, y: dy } = ctx.input(|i| i.pointer.delta());
            // TODO: we need the plot span here ... maybe
            match modifiers {
                // Alt key is pressed → change xoffset.
                [true, false, false] => {
                    file.properties.xoffset += 0.001 * (dx as f64);
                }
                // Ctrl key is pressed → change yoffset.
                [false, true, false] => {
                    file.properties.yoffset -= 0.001 * (dy as f64);
                }
                // Shift is pressed → change yscale.
                [false, false, true] => {
                    let yscale = file.properties.yscale;
                    file.properties.yscale -= yscale * 0.01 * (dy as f64);
                }
                // If several modifiers are pressed at the same time,
                // we ignore the input.
                _ => (),
            }
            log::debug!("file offset: {}", file.properties.yoffset);
        }
    }
}
