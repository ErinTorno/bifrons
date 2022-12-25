use bevy::{asset::*, reflect::TypeUuid};
use bevy_egui::egui;

#[derive(TypeUuid)]
#[uuid = "f0b75841-3643-4e9b-9d85-4e828417d7b0"]
pub struct Font(pub egui::FontData);

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let font = Font(egui::FontData::from_owned(bytes.to_vec()));
            load_context.set_default_asset(LoadedAsset::new(font));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["otf", "ttf"]
    }
}