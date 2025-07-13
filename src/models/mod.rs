pub mod activity;
pub mod actor;
pub mod object;

// Re-export commonly used types
pub use actor::{Actor, PublicKey, Icon};
pub use activity::Activity;
pub use object::{Note, OrderedCollection};

// Conditional re-exports based on target
#[cfg(feature = "native")]
pub use actor::Actor as NativeActor;

#[cfg(feature = "esp32")]
pub use actor::Actor as EmbeddedActor;
