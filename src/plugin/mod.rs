pub use self::meme::MemePlugin;
pub use self::viper::ViperPlugin;

mod meme;
mod viper;

/// All plugins must implement the `Plugin` trait. A plugin is a user defined
/// bot function that handles certain messages it receives.
pub trait Plugin: Send + ::std::fmt::Debug {
    /// Creates a new `Plugin` in a `Box` container.
    fn new() -> Box<Plugin> where Self: Sized;

    /// Returns true if the plugin should call its handler, and false
    /// otherwise.
    fn is_match(&self, message: &::Message) -> bool;

    /// Performs the plugin action.
    fn handle(&mut self, message: &::Message);
}
