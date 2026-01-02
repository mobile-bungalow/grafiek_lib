use grafiek_engine::{Engine, NodeIndex};

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
mod web {
    use grafiek_engine::NodeIndex;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    pub struct PendingUpload {
        pub node_idx: NodeIndex,
        pub slot: usize,
        pub width: u32,
        pub height: u32,
        pub data: Vec<u8>,
    }

    thread_local! {
        pub static PENDING_UPLOADS: RefCell<Vec<PendingUpload>> = const { RefCell::new(Vec::new()) };
    }

    pub fn pick_and_load_image(node_idx: NodeIndex, slot: usize) {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");

        let input: web_sys::HtmlInputElement = document
            .create_element("input")
            .expect("failed to create input")
            .dyn_into()
            .expect("not an input element");

        input.set_type("file");
        input.set_accept("image/png,image/jpeg,image/gif,image/webp,image/bmp");

        let input_clone = input.clone();
        let onchange = Closure::once(Box::new(move || {
            let Some(files) = input_clone.files() else {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };

            let reader = web_sys::FileReader::new().expect("failed to create FileReader");
            let reader_clone = reader.clone();

            let onload = Closure::once(Box::new(move || {
                let result = reader_clone.result().expect("no result");
                let array = js_sys::Uint8Array::new(&result);
                let bytes = array.to_vec();

                match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.into_rgba8();
                        let (w, h) = rgba.dimensions();
                        PENDING_UPLOADS.with(|uploads| {
                            uploads.borrow_mut().push(PendingUpload {
                                node_idx,
                                slot,
                                width: w,
                                height: h,
                                data: rgba.into_raw(),
                            });
                        });
                    }
                    Err(e) => log::error!("Failed to decode image: {e}"),
                }
            }));

            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();

            reader
                .read_as_array_buffer(&file)
                .expect("failed to read file");
        }));

        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();

        input.click();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn pick_and_load_image(_engine: &mut Engine, node_idx: NodeIndex, slot: usize) {
    web::pick_and_load_image(node_idx, slot);
}

#[cfg(target_arch = "wasm32")]
pub fn process_pending_uploads(engine: &mut Engine) {
    web::PENDING_UPLOADS.with(|uploads| {
        for upload in uploads.borrow_mut().drain(..) {
            if let Err(e) = engine.upload_texture(
                upload.node_idx,
                upload.slot,
                upload.width,
                upload.height,
                &upload.data,
            ) {
                log::error!("Failed to upload texture: {e}");
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn process_pending_uploads(_engine: &mut Engine) {
    // No-op on native
}
