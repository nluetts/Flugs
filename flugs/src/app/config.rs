use app_core::string_error::ErrorStringExt;
use std::{
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug)]
pub struct Config {
    pub search_path: PathBuf,
    pub svg_width: u64,
    pub svg_height: u64,
    pub x_label: String,
    pub y_label: String,
    pub x_ticks: TicksInput,
    pub y_ticks: TicksInput,
    pub num_x_minorticks: usize,
    pub num_y_minorticks: usize,
    pub draw_xaxis: bool,
    pub draw_yaxis: bool,
}

impl Default for Config {
    fn default() -> Self {
        let search_path = PathBuf::from("/tmp/");
        let svg_width = 800;
        let svg_height = 600;
        let x_label = "x-label".to_string();
        let y_label = "y-label".to_string();

        Self {
            search_path,
            svg_width,
            svg_height,
            x_label,
            y_label,
            draw_xaxis: true,
            draw_yaxis: true,
            x_ticks: Default::default(),
            y_ticks: Default::default(),
            num_x_minorticks: 0,
            num_y_minorticks: 0,
        }
    }
}

impl Config {
    pub fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Preferences");
        ui.separator();

        ui.label("Search Path");
        let mut path = self.search_path.to_string_lossy();
        ui.text_edit_singleline(&mut path);
        self.search_path = path.to_string().into();

        ui.label("Width of exported SVG");
        ui.add(egui::DragValue::new(&mut self.svg_width).speed(10));
        ui.label("Height of exported SVG");
        ui.add(egui::DragValue::new(&mut self.svg_height).speed(10));
        ui.checkbox(&mut self.draw_xaxis, "Draw X-Axis");
        ui.label("X-Label");
        ui.text_edit_singleline(&mut self.x_label);
        ui.label("Number of Minor X-Ticks");
        ui.add(egui::DragValue::new(&mut self.num_x_minorticks).range(0..=9));
        ui.checkbox(&mut self.draw_yaxis, "Draw Y-Axis");
        ui.label("Y-Label");
        ui.text_edit_singleline(&mut self.y_label);
        ui.label("Number of Minor Y-Ticks");
        ui.add(egui::DragValue::new(&mut self.num_y_minorticks).range(0..=9));

        ui.separator();

        if ui.button("Save to Config File").clicked() {
            if let Err(e) = self.to_config_file() {
                log::error!("{e}");
            };
        }

        ui.separator();
        ui.heading("Temporary Settings");
        ui.separator();

        ui.label("X-Ticks");
        if egui::TextEdit::singleline(&mut self.x_ticks.raw)
            .text_color(self.x_ticks.color)
            .show(ui)
            .response
            .changed()
        {
            let _ = self.x_ticks.parse(ctx);
        };
        ui.label("Y-Ticks");
        if egui::TextEdit::singleline(&mut self.y_ticks.raw)
            .text_color(self.y_ticks.color)
            .show(ui)
            .response
            .changed()
        {
            let _ = self.y_ticks.parse(ctx);
        };
    }
}

impl Config {
    pub fn from_config_file() -> Result<Self, String> {
        let mut config = Self::default();
        #[allow(deprecated)]
        let Some(home) = std::env::home_dir() else {
            return Err("could not determine home directory to load config file".into());
        };
        let config_raw = {
            let path = home.join(PathBuf::from(".flugs"));
            let mut file = std::fs::File::open(path).err_to_string("could not open config file")?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .err_to_string("could not load config file")?;
            buf
        };
        for line in config_raw.lines() {
            // Lines starting with "#" are considered comments.
            if line.starts_with("#") {
                continue;
            }
            let mut iter = line.split("=");
            let key = iter.next();
            let val = iter.next();
            match (key, val) {
                (Some("search_path"), Some(path_str)) => {
                    let path = PathBuf::from_str(path_str)
                        .expect("could not parse 'search_path' as directory name");
                    config.search_path = path;
                }
                (Some("svg_width"), Some(width_str)) => {
                    if let Ok(width) = width_str.parse::<u64>() {
                        config.svg_width = width;
                    } else {
                        log::warn!("could not parse 'svg_width' as number")
                    }
                }
                (Some("svg_height"), Some(height_str)) => {
                    if let Ok(height) = height_str.parse::<u64>() {
                        config.svg_height = height;
                    } else {
                        log::warn!("could not parse 'svg_height' as number")
                    }
                }
                (Some("draw_xaxis"), Some("true")) => {
                    config.draw_xaxis = true;
                }
                (Some("draw_xaxis"), Some("false")) => {
                    config.draw_xaxis = false;
                }
                (Some("x_label"), Some(x_label)) => {
                    config.x_label = x_label.to_string();
                }
                (Some("draw_yaxis"), Some("true")) => {
                    config.draw_yaxis = true;
                }
                (Some("draw_yaxis"), Some("false")) => {
                    config.draw_yaxis = false;
                }
                (Some("y_label"), Some(y_label)) => {
                    config.y_label = y_label.to_string();
                }
                (Some("num_x_minorticks"), Some(num_str)) => {
                    if let Ok(num) = num_str.parse::<usize>() {
                        config.num_x_minorticks = num;
                    }
                }
                (Some("num_y_minorticks"), Some(num_str)) => {
                    if let Ok(num) = num_str.parse::<usize>() {
                        config.num_y_minorticks = num;
                    }
                }
                _ => continue,
            }
        }
        Ok(config)
    }

    fn to_config_file(&self) -> Result<(), String> {
        #[allow(deprecated)]
        let Some(config_file_path) =
            std::env::home_dir().map(|path| path.join(PathBuf::from(".flugs")))
        else {
            return Err("could open config file".into());
        };

        log::info!("attempting to save config to {config_file_path:?}");

        let mut config_file = match std::fs::File::create(config_file_path) {
            Ok(file) => file,
            Err(err) => return Err(format!("could not open config file: {err}")),
        };

        let mut wrt_results = Vec::with_capacity(10);

        wrt_results.push(config_file.write_all(
            &format!("search_path={}\n", self.search_path.to_string_lossy()).into_bytes(),
        ));
        wrt_results
            .push(config_file.write_all(&format!("svg_width={}\n", self.svg_width).into_bytes()));
        wrt_results
            .push(config_file.write_all(&format!("svg_height={}\n", self.svg_height).into_bytes()));
        wrt_results
            .push(config_file.write_all(&format!("x_label={}\n", self.x_label).into_bytes()));
        wrt_results
            .push(config_file.write_all(&format!("y_label={}\n", self.y_label).into_bytes()));
        wrt_results.push(
            config_file
                .write_all(&format!("num_x_minorticks={}\n", self.num_x_minorticks).into_bytes()),
        );
        wrt_results.push(
            config_file
                .write_all(&format!("num_y_minorticks={}\n", self.num_y_minorticks).into_bytes()),
        );
        wrt_results.push(
            config_file.write_all(
                &format!(
                    "draw_xaxis={}\n",
                    if self.draw_xaxis { "true" } else { "false" }
                )
                .into_bytes(),
            ),
        );
        wrt_results.push(
            config_file.write_all(
                &format!(
                    "draw_yaxis={}\n",
                    if self.draw_yaxis { "true" } else { "false" }
                )
                .into_bytes(),
            ),
        );

        for res in wrt_results {
            if let Err(e) = res {
                return Err(format!("could not write to config file: {e}"));
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct TicksInput {
    pub pos: Vec<f64>,
    raw: String,
    color: egui::Color32,
}

impl TicksInput {
    fn parse(&mut self, ctx: &egui::Context) -> Result<(), ()> {
        let res = self.parse_inner();
        if res.is_err() {
            self.color = egui::Color32::RED;
        } else {
            self.color = if ctx.theme() == egui::Theme::Light {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            };
        }
        Ok(())
    }

    fn parse_inner(&mut self) -> Result<(), ()> {
        self.pos.drain(..);
        for s in self.raw.split(" ") {
            // Handle input of range (syntax is "start:stop:step").
            if s.contains(":") {
                let mut parts = s.split(":");
                let mut current = parts.next().and_then(|s| s.parse::<f64>().ok()).ok_or(())?;
                let step = parts.next().and_then(|s| s.parse::<f64>().ok()).ok_or(())?;
                let stop = parts.next().and_then(|s| s.parse::<f64>().ok()).ok_or(())?;
                // Handle a couple of wrong inputs, such as wrong sign of step ...
                if (stop - current).is_sign_positive() && step.is_sign_negative() {
                    return Err(());
                }
                if (stop - current).is_sign_negative() && step.is_sign_positive() {
                    return Err(());
                }
                // ... or no step at all.
                if step == 0.0 {
                    return Err(());
                }
                while current <= stop {
                    self.pos.push(current);
                    current += step;
                }
            // Handle specific numbers.
            } else {
                let x = s.parse::<f64>().map_err(|_| ())?;
                self.pos.push(x);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config_file() {
        #[allow(unused)]
        let res = Config::from_config_file();
        // dbg!(res);
    }
}
