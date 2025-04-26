#[derive(Debug, Clone)]
pub enum BuildSource {
    Main,
    Example(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BundleType {
    WinVST3,
    MacVST3,
    AUv3,
    WinStandalone,
    MacStandalone,
}

#[derive(clap::Parser)]
pub struct Settings {
    wrapper_formats: Option<Vec<BundleType>>,
}
