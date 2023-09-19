pub mod camera;
pub mod figure;
pub mod terrain;

// biblioteca
use vek::*;
use dot_vox;

// projeto
use common::figure::Segment;
use client::Client;

// caixote
use crate::{
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

        character::{
            CharacterSkeleton,
            RunAnimation
        }
    }
};

// local
use self::{
    camera::Camera,
    figure::Figure,
    terrain::Terrain
};

// TODO: não dificultar esse código
const CURSOR_PAN_SCALE: f32 = 0.005;

struct Skybox {
    model: Model<SkyboxPipeline>,
    locals: Consts<SkyboxLocals>
}

pub struct Scene {
    globals: Consts<Globals>,
    camera: Camera,
    
    skybox: Skybox,
    terrain: Terrain,

    test_figure: Figure<CharacterSkeleton>
}

// TODO: fazer um asset proper para carregar o sistema
fn load_segment(filename: &'static str) -> Segment {
    Segment::from(dot_vox::load(&(concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/").to_string() + filename)).unwrap())
}

impl Scene {
    /// criar um novo `scene` com parâmetros padrões
    pub fn new(renderer: &mut Renderer, client: &Client) -> Self {
        Self {
            globals: renderer
                .create_consts(&[Globals::default()])
                .unwrap(),

            camera: Camera::new(),

            skybox: Skybox {
                model: renderer
                    .create_model(&create_skybox_mesh())
                    .unwrap(),

                locals: renderer
                    .create_consts(&[SkyboxLocals::default()])
                    .unwrap()
            },

            terrain: Terrain::new(),

            test_figure: Figure::new(
                renderer, [
                    Some(load_segment("head.vox").generate_mesh_with_offset(Vec3::new(-7.0, -5.5, -1.0))),
                    Some(load_segment("chest.vox").generate_mesh_with_offset(Vec3::new(-6.0, -3.0, 0.0))),
                    Some(load_segment("belt.vox").generate_mesh_with_offset(Vec3::new(-5.0, -3.0, 0.0))),
                    Some(load_segment("pants.vox").generate_mesh_with_offset(Vec3::new(-5.0, -3.0, 0.0))),
                    
                    Some(load_segment("hand.vox").generate_mesh_with_offset(Vec3::new(-2.0, -2.0, -1.0))),
                    Some(load_segment("hand.vox").generate_mesh_with_offset(Vec3::new(-2.0, -2.0, -1.0))),
                    
                    Some(load_segment("foot.vox").generate_mesh_with_offset(Vec3::new(-2.5, -3.0, -2.0))),
                    Some(load_segment("foot.vox").generate_mesh_with_offset(Vec3::new(-2.5, -3.0, -2.0))),
                    
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
                .unwrap()
        }
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
    pub fn maintain_gpu_data(&mut self, renderer: &mut Renderer, client: &Client) {
        // computar matrizes de câmera
        let (view_mat, proj_mat, cam_pos) = self.camera.compute_dependents();

        // atualizar constantes globais
        renderer.update_consts(&mut self.globals, &[Globals::new(
            view_mat,
            proj_mat,
            cam_pos,
            
            self.camera.get_focus_pos(),

            10.0,
            
            client.state().get_time_of_day(),
            client.state().get_tick()
        )])
            .expect("falha ao atualizar constantes globais");

        // manter dados gpu do terreno
        self.terrain.maintain_gpu_data(renderer);

        // TODO: não fazer isso aqui
        RunAnimation::update_skeleton(
            &mut self.test_figure.skeleton,

            client.stare().get_tick()
        );

        self.test_figure.update_locals(renderer, FigureLocals::default()).unwrap();
        self.test_figure.update_skeleton(renderer).unwrap();
    }

    /// renderizar cena usando o `renderer` fornecido
    pub fn render_to(&self, renderer: &mut Renderer) {
        // renderizar skybox primeiramente
        
        renderer.render_skybox(
            &self.skybox.model,
            &self.globals,
            &self.skybox.locals
        );

        // renderizar terreno
        self.terrain.render(renderer, &self.globals);

        // renderizar o teste de figura
        self.test_figure.render(renderer, &self.globals);
    }
}
