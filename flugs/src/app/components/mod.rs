mod file_handling;
mod plotter;
mod search;

pub use file_handling::{File, FileHandler};
pub(in crate::app) use file_handling::{FileID, FileProperties, Group};
pub use plotter::{save_svg, Plotter};
pub use search::Search;
