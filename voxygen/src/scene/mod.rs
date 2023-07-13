pub mod camera;

// caixote
use crate::render::{
    Consts,
    Globals,
    Model,
    Renderer,
    SkyboxPipeline,
    SkyboxLocals,

    create_skybox_mesh
};

// local
use self::camera::Camera;

struct Skybox {
    model: Model<SkyboxPipeline>,
    locals: Consts<SkyboxLocals>
}

pub struct Scene {
    camera: Camera,
    globals: Consts<Globals>,
    skybox: Skybox
}

impl Scene {
    /// criar um novo `scene` com parâmetros padrões
    pub fn new(renderer: &mut Renderer) -> Self {
        Self {
            camera: Camera::new(),

            globals: renderer
                .create_consts_with(Globals::new())
                .unwrap(),

            skybox: Skybox {
                model: renderer
                    .create_model(&create_skybox_mesh())
                    .unwrap(),

                locals: renderer
                    .create_consts_with(SkyboxLocals::new())
                    .unwrap()
            }
        }
    }

    /// renderizar cena usando o `renderer` fornecido
    pub fn render_to(&self, renderer: &mut Renderer) {
        // renderizar skybox primeiramente
        
        renderer.render_skybox(
            &self.skybox.model,
            &self.skybox.locals,
            &self.globals
        );
    }
}