mod connectivity;
#[cfg(feature = "desktop-session")]
mod geo;
#[cfg(feature = "desktop-session")]
mod speech;

use std::sync::Arc;

use bridgething_companion::api::ModelPlatform;

#[cfg(feature = "desktop-session")]
use crate::backends::geo::Locator;
use crate::backends::{ModelPaths, Platform, portable::PortableScaler, voice};

pub fn platform() -> Platform {
  let paths = ModelPaths::default();
  let voice = voice(&paths, ModelPlatform::Windows);
  Platform {
    #[cfg(feature = "desktop-session")]
    geo: Some(Arc::new(Locator::new(geo::run))),
    #[cfg(not(feature = "desktop-session"))]
    geo: None,
    notifications: None,
    #[cfg(feature = "desktop-session")]
    audio: Some(Arc::new(speech::WinRtAudio::new())),
    #[cfg(not(feature = "desktop-session"))]
    audio: None,
    connectivity: Some(Arc::new(connectivity::NetworkInformationConnectivity::default())),
    image: Some(Arc::new(PortableScaler)),
    speech: voice.speech,
    nlu: voice.nlu,
    model_validator: voice.model_validator,
    model_platform: voice.model_platform,
    models: paths,
  }
}
