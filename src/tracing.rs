use anyhow::Result;
use tracing_subscriber;
use tracing_subscriber::fmt;

pub struct Tracing;

impl Tracing {
    pub fn init() -> Result<()> {
        let format = fmt::format().with_level(true).with_target(false).compact();
        tracing_subscriber::fmt().event_format(format).init();
        Ok(())
    }
}
