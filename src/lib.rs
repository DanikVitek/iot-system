use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::{SubscriberInitExt, TryInitError},
    EnvFilter,
};

pub mod config;
pub mod domain;

#[cfg(feature = "tonic")]
pub mod proto {
    tonic::include_proto!("iot_system");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("iot_system_descriptor");
}

#[inline(always)]
pub fn setup_tracing(
    logs_dir: impl AsRef<Path>,
    logs_file_name: impl AsRef<Path>,
) -> Result<WorkerGuard, TryInitError> {
    _setup_tracing(logs_dir.as_ref(), logs_file_name.as_ref())
}

fn _setup_tracing(logs_dir: &Path, logs_file_name: &Path) -> Result<WorkerGuard, TryInitError> {
    let file_appender = tracing_appender::rolling::never(logs_dir, logs_file_name);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_thread_ids(true)
        .finish()
        .with(tracing_subscriber::fmt::layer().with_writer(file_writer))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .try_init()?;

    Ok(guard)
}

/// A trait for applying Kotlin-like convenience methods to types.
pub trait KtConvenience: Sized {
    #[inline]
    fn apply(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }

    #[inline]
    fn try_apply<E>(mut self, f: impl FnOnce(&mut Self) -> Result<(), E>) -> Result<Self, E> {
        f(&mut self)?;
        Ok(self)
    }

    #[inline]
    fn r#let<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }

    #[inline]
    fn take_if(self, predicate: impl FnOnce(&Self) -> bool) -> Option<Self> {
        if predicate(&self) {
            Some(self)
        } else {
            None
        }
    }

    #[inline]
    fn also(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}

// Implement the trait for all types.
impl<T> KtConvenience for T {}

/// A macro for cloning variables. Useful for moving
/// variables into closures, like `Rc` or `Arc`.
#[macro_export]
macro_rules! reclone {
    ($v:ident $(,)?) => {
        let $v = $v.clone();
    };
    (mut $v:ident $(,)?) => {
        let mut $v = $v.clone();
    };
    ($($v:ident),+ $(,)?) => {
        $(
            let $v = $v.clone();
        )+
    };
}
