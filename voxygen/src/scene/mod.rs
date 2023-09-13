pub mod camera;

// padrão
use std::time::Duration;

// biblioteca
use vek::*;

// projeto
use client::{
    self,
    Client
};

// caixote
use crate::{
    Error,
    
    render::{
        Consts,
        Globals,
        Model,
        Renderer,
        SkyboxPipeline,
        SkyboxLocals,

        create_skybox_mesh
    },

    window::Event
};

// local
use self::camera::Camera;

struct Skybox {
    model: Model<SkyboxPipeline>,
    locals: Consts<SkyboxLocals>
}

// todo: não forçar código disso
const CURSOR_PAN_SCALE: f32 = 0.005;

pub struct Scene {
    camera: Camera,
    globals: Consts<Globals>,
    skybox: Skybox,
    client: Client
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
            },

            client: Client::new()
        }
    }

    /// ticka a cena (e o client anexado nela)
    pub fn tick(&mut self, dt: Duration) -> Result<(), Error> {
        self.client.tick(client::Input {}, dt)?;

        Ok(())
    }

    /// auxilia um evento de input de usuário que está sendo recebido (exemplos: cursor movendo, tecla pressionada, janela fechada, etc.)
    pub fn handle_input_event(&mut self, event: Event) -> bool {
        match event {
            // paralizar o cursor faz com que a câmera rotacione
            Event::CursorPan(delta) => {
                self.camera.rotate_by(Vec3::from(delta) * CURSOR_PAN_SCALE);

                true
            },

            // todos os outros eventos não-auxiliados
            _ => false
        }
    }

    /// mantém e atualiza dados da gpu como buffers constantes, modelos, etc.
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
            self.client.state().get_time_of_day(),
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
