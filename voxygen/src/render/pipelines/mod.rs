pub mod figure;
pub mod skybox;
pub mod terrain;
pub mod ui;

// biblioteca
use gfx::{
    self,

    // macros
    gfx_defines,
    gfx_constant_struct_meta,
    gfx_impl_struct_meta
};

use vek::*;

// local
use super::util::arr_to_mat;

gfx_defines! {
    constant Globals {
        view_mat: [[f32; 4]; 4] = "view_mat",
        proj_mat: [[f32; 4]; 4] = "proj_mat",

        cam_pos: [f32; 4] = "cam_pos",
        focus_pos: [f32; 4] = "focus_pos",
        // TODO: corrigir qualquer erro de alinhamento necessita desses uniformes para serem alinhados
        
        view_distance: [f32; 4] = "view_distance",
        time_of_day: [f32; 4] = "time_of_day", // TODO: fazer disso um f64
        tick: [f32; 4] = "tick"
    }
}

impl Globals {
    /// cria novas constantes globais com valores padrões
    pub fn default() -> Self {
        Self {
            view_mat: arr_to_mat(Mat4::identity().into_col_array()),
            proj_mat: arr_to_mat(Mat4::identity().into_col_array()),

            cam_pos: [0.0; 4],
            focus_pos: [0.0; 4],
            
            view_distance: [0.0; 4],
            time_of_day: [0.0; 4],
            tick: [0.0; 4]
        }
    }

    /// cria novas constantes globais por meio dos parâmetros fornecidos
    pub fn new(
        view_mat: Mat4<f32>,
        proj_mat: Mat4<f32>,

        cam_pos: Vec3<f32>,
        focus_pos: Vec3<f32>,

        view_distance: f32,
        time_of_day: f32,
        tick: f64
    ) -> Self {
        Self {
            view_mat: arr_to_mat(view_mat.into_col_array()),
            proj_mat: arr_to_mat(proj_mat.into_col_array()),

            cam_pos: Vec4::from(cam_pos).into_array(),
            focus_pos: Vec4::from(focus_pos).into_array(),

            view_distance: [view_distance; 4],
            time_of_day: [time_of_day as f32; 4],
            tick: [tick as f32; 4]
        }
    }
}
