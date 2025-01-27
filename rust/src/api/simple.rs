use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use flume::{bounded, Receiver, Sender};
use irondash_engine_context::EngineContext;
use irondash_run_loop::RunLoop;
use irondash_texture::{
    BoxedPixelData, PayloadProvider, SendableTexture, SimplePixelData, Texture,
};
use log::{error, info};
use simple_logger::SimpleLogger;

use crate::core::{get_images, my_glium::create_ogl_texture, GlSource};

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    SimpleLogger::new().init().unwrap();
    info!("Initializing app");
    // Default utilities - feel free to custom
    flutter_rust_bridge::setup_default_user_utils();
}

lazy_static::lazy_static! {

    static ref TEXTURES_REGISTRY: Mutex<HashMap<i64, Arc<SendableTexture<BoxedPixelData>>>> = Mutex::new(HashMap::new());
}

pub fn create_that_texture_please(engine_handle: i64) -> anyhow::Result<i64> {
    let (tx_texture_id, rx_texture_id) = bounded(1);

    RunLoop::sender_for_main_thread().unwrap().send(move || {
        if let Err(e) = EngineContext::get() {
            error!("Failed to get engine handle: {:?}", e);
        }
        let (tx, rx) = bounded(2);
        let provider = Arc::new(GlSource { rx });

        let texture = Texture::new_with_provider(engine_handle, provider).unwrap();
        let texture_id = texture.id();
        tx_texture_id
            .send(texture_id)
            .expect("Failed to send texture");

        let texture_arc = texture.into_sendable_texture();
        {
            let _ = TEXTURES_REGISTRY
                .lock()
                .unwrap()
                .insert(texture_id, texture_arc.clone());
        }

        std::thread::spawn(move || {
            get_images(tx, texture_arc).expect("Failed to get images");
        });
    });
    Ok(rx_texture_id.recv().unwrap())
}


pub fn get_opengl_texture(engine_handle: i64) -> anyhow::Result<i64> {
    let (tx_texture_id, rx_texture_id) = bounded(1);

    RunLoop::sender_for_main_thread().unwrap().send(move || {

      let a =  create_ogl_texture(engine_handle);
      info!("sending texture id");
      tx_texture_id
          .send(a.unwrap())
          .expect("Failed to send texture");
    }
    );
    info!("waiting for texture id");
    Ok(rx_texture_id.recv().unwrap())

}