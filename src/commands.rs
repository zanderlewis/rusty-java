use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Build the Java project
    Build,
    /// Run the Java project
    Run,
    /// Clean the build output
    Clean,
    /// Initialize a RSJ project
    Init,
}
