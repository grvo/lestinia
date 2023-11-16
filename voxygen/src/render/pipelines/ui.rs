use gfx::{
    self,

    // macros
    gfx_defines,

    gfx_vertex_struct_meta,
    gfx_impl_struct_meta,

    gfx_pipeline,
    gfx_pipeline_inner
};

use vek::*;

use super::super::{
    Pipeline,

    TgtColorFmt,
    TgtDepthFmt,

    Mesh,
    Quad
};

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "v_pos",
        uv: [f32; 2] = "v_uv"
    }

    constant Locals {
        color: [f32; 4] = "v_color",
		mode: u32 = "v_mode"
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        tex: gfx::TextureSampler<[f32; 4]> = "u_tex",

		scissor: gfx::Scissor = (),

        tgt_color: gfx::BlendTarget<TgtColorFmt> = ("tgt_color", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        tgt_depth: gfx::DepthTarget<TgtDepthFmt> = gfx::preset::depth::PASS_TEST
    }
}

pub struct UiPipeline;

impl Pipeline for UiPipeline {
    type Vertex = Vertex;
}

/// desenhar texto pela cache de textura do texto `tex` no shader de fragmento
pub const MODE_TEXT: u32 = 0;

/// desenhar uma imagem pela textura em `tex` no shader de fragmento
pub const MODE_IMAGE: u32 = 1;

/// ignorar `tex` e desenhar simple, geometria 2d colorida
pub const MODE_GEOMETRY: u32 = 2;

pub enum Mode {
	Text,
	Image,
	Geometry
}

impl Mode {
	fn value(self) -> u32 {
		match Self {
			Mode::Text => MODE_TEXT,
			Mode::Image => MODE_IMAGE,
			Mode::Geometry => MODE_GEOMETRY
		}
	}
}

pub fn push_quad_to_mesh(mesh: &mut Mesh<UiPipeline>, rect: Aabr<f32>, uv_rect: Aabr<f32>, color: [f32; 4], mode: Mode) {
	let mode_val = mode.value();

	let v = |pos, uv| {
		Vertex {
			pos,
			uv,
			color,

			mode: mode_val
		}
	};

	let aabr_to_lbrt = |aabr: Aabr<f32>| (
		aabr.min.x, aabr.min.y,
		aabr.max.x, aabr.max.y
	);

	let (l, b, r, t) = aabr_to_lbrt(rect);
	let (uv_l, uv_b, uv_r, uv_t) = aabr_to_lbrt(uv_rect);

	mesh.push_quad(Quad::new(
		v([r, t], [uv_r, uv_t]),
		v([l, t], [uv_l, uv_t]),
		v([l, b], [uv_l, uv_b]),
		v([r, b], [uv_r, uv_b])
	));
}
