use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Width of the simulation
    #[arg(long, default_value_t = 1024)]
    width: u32,
    /// Height of the simulation
    #[arg(long, default_value_t = 1024)]
    height: u32,
}

impl Config {
    /// Returns dimensions of the simulation.
    #[inline]
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
