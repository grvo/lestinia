pub mod camera;
pub mod figure;

// padrão
use std::time::Duration;

// biblioteca
use vek::*;
use dot_vox;

// projeto
use client::{
    self,
    Client
};

use common::figure::Segment;

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
        FigureLocals,

        create_skybox_mesh
    },

    window::Event,
    mesh::Meshable,

    anim::{
        Animation,
        CharacterSkeleton,
        RunAnimation
    }
};

// local
use self::{
    camera::Camera,
    figure::Figure
};

// TODO: não dificultar esse código
const CURSOR_PAN_SCALE: f32 = 0.005;

struct Skybox {
    model: Model<SkyboxPipeline>,
    locals: Consts<SkyboxLocals>
}

pub struct Scene {
    camera: Camera,
    globals: Consts<Globals>,
    skybox: Skybox,

    test_figure: Figure<CharacterSkeleton>,
    
    client: Client
}

// TODO: fazer um asset proper para carregar o sistema
fn load_segment(filename: &'static str) -> Segment {
    Segment::from(dot_vox::load(&(concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/").to_string() + filename)).unwrap())
}

impl Scene {
    /// criar um novo `scene` com parâmetros padrões
    pub fn new(renderer: &mut Renderer) -> Self {
        Self {
            camera: Camera::new(),

            globals: renderer
                .create_consts(&[Globals::default()])
                .unwrap(),

            skybox: Skybox {
                model: renderer
                    .create_model(&create_skybox_mesh())
                    .unwrap(),

                locals: renderer
                    .create_consts(&[SkyboxLocals::default()])
                    .unwrap()
            },

            test_figure: Figure::new(
                renderer, [
                    Some(load_segment("head.vox").generate_mesh_with_offset(Vec3::new(-7.0, -5.5, -1.0))),
                    Some(load_segment("chest.vox").generate_mesh_with_offset(Vec3::new(-6.0, -3.0, 0.0))),
                    Some(load_segment("belt.vox").generate_mesh_with_offset(Vec3::new(-5.0, -3.0, 0.0))),
                    Some(load_segment("pants.vox").generate_mesh_with_offset(Vec3::new(-5.0, -3.0, 0.0))),
                    Some(load_segment("hand.vox").generate_mesh_with_offset(Vec3::new(-2.0, -2.0, -1.0))),
                    Some(load_segment("hand.vox").generate_mesh_with_offset(Vec3::new(-2.0, -2.0, -1.0))),
                    Some(load_segment("foot.vox").generate_mesh_with_offset(Vec3::new(-2.5, -3.0, 0.0))),
                    Some(load_segment("foot.vox").generate_mesh_with_offset(Vec3::new(-2.5, -3.0, 0.0))),
                    Some(load_segment("sword.vox").generate_mesh_with_offset(Vec3::new(-6.5, -1.0, 0.0))),

                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None
                ],

                CharacterSkeleton::new()
            )
                .unwrap(),

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
        renderer.update_consts(&mut self.globals, &[Globals::new(
            view_mat,
            proj_mat,
            cam_pos,
            
            self.camera.get_focus_pos(),

            10.0,
            self.client.state().get_time_of_day(),
            0.0
        )])
            .expect("falha ao atualizar constantes globais");

        // TODO: não fazer isso aqui
        RunAnimation::update_skeleton(
            &mut self.test_figure.skeleton,

            self.client.stare().get_tick()
        );

        self.test_figure.update_locals(renderer, FigureLocals::default());
        self.test_figure.update_skeleton(renderer);
    }

    /// renderizar cena usando o `renderer` fornecido
    pub fn render_to(&self, renderer: &mut Renderer) {
        // renderizar skybox primeiramente
        
        renderer.render_skybox(
            &self.skybox.model,
            &self.globals,
            &self.skybox.locals
        );

        // renderizar o teste de figura
        self.test_figure.render(renderer, &self.globals);
    }
}
