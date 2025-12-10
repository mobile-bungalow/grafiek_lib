use grafiek_engine::{Engine, NodeIndex};

pub fn pick_and_load_image(engine: &mut Engine, node_idx: NodeIndex, slot: usize) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
        .pick_file()
    else {
        return;
    };

    let img = match image::open(&path) {
        Ok(img) => img.into_rgba8(),
        Err(e) => return log::error!("Failed to load image {path:?}: {e}"),
    };

    let (w, h) = img.dimensions();
    if let Err(e) = engine.upload_texture(node_idx, slot, w, h, &img.into_raw()) {
        log::error!("Failed to upload texture: {e}");
    }
}
