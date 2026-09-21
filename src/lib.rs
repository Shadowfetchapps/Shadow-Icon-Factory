pub mod error;
pub mod generate;
pub mod paths;
pub mod settings;
pub mod spec;

pub use error::{Error, Result};
pub use generate::{generate, load_master, GenerateOptions, GenerateReport};
pub use settings::Theme;
pub use spec::Target;
