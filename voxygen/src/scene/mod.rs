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
                .create_consts_with(Globals::default())
                .unwrap(),

            skybox: Skybox {
                model: renderer
                    .create_model(&create_skybox_mesh())
                    .unwrap(),

                locals: renderer
                    .create_consts_with(SkyboxLocals::default())
                    .unwrap()
            }
        }
    }

    pub fn maintain_gpu_data(&mut self, renderer: &mut Renderer) {
        // computar matrizes de câmera
        let (view_mat, proj_mat, cam_pos) = self.camera.compute_dependents();

        // atualizar constantes globais
        renderer.update_consts(&mut self.globals, Globals::new(
            view_mat,
            proj_mat,
            cam_pos,
            
            self.camera.get_focus_pos(),

            10.0,
            0.0,
            0.0
        ))
            .expect("falha ao atualizar constantes globais");
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