pub mod character;
pub mod skybox;

// biblioteca
use gfx::{
    self,

    // macros
    gfx_defines,
    gfx_vertex_struct_meta,
    gfx_constant_struct_meta,
    gfx_impl_struct_meta,
    gfx_pipeline,
    gfx_pipeline_inner
};

gfx_defines! {
    constant Globals {
        view_mat: [[f32; 4]; 4] = "view_mat",
        proj_mat: [[f32; 4]; 4] = "proj_mat",

        cam_pos: [f32; 4] = "cam_pos",
        focus_pos: [f32; 4] = "focus_pos",
        
        view_distance: [f32; 4] = "view_distance",
        tod: [f32; 4] = "tod",
        time: [f32; 4] = "time"
    }
}