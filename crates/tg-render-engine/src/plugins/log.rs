use crate::{App, Plugin};
use specs::WorldExt;
use tracing::{
    debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
    Level,
};
use tracing_log::LogTracer;
#[cfg(feature = "tracing-chrome")]
use tracing_subscriber::fmt::{format::DefaultFields, FormattedFields};
use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter, fmt};
#[derive(Default)]
pub struct LogPlugin;

/// `LogPlugin` settings
#[derive(Clone)]
pub struct LogSettings {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: Level,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            filter: "wgpu=error".to_string(),
            level: Level::INFO,
        }
    }
}

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        let default_filter = {
            let settings = LogSettings::default();
            let new_setting = settings.clone();
            app.world.insert(settings);
            format!("{},{}", new_setting.level, new_setting.filter)
        };
        LogTracer::init().unwrap();
        let filter_layer = EnvFilter::try_from_default_env()
            // .or_else(|_| EnvFilter::try_new(&default_filter))
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();
        let subscriber = Registry::default().with(filter_layer).with(fmt::layer().pretty());
        tracing::subscriber::set_global_default(subscriber)
                .expect("Could not set global default tracing subscriber. If you've already set up a tracing subscriber, please disable LogPlugin from Bevy's DefaultPlugins");
    }
}
