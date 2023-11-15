// padrão
use std::marker::PhantomData;

// biblioteca
use gfx::{
    self,

    traits::Factory
};

use image::{
    DynamicImage,
    GenericImageView
};

// local
use super::{
    RenderError,
    Pipeline,
    gfx_backend
};

type ShaderFormat = (gfx::format::R8_G8_B8_A8, gfx::format::Srgb);

/// representa uma imagem que foi postada na gpu
pub struct Texture<P: Pipeline> {
    pub tex: gfx::handle::Texture<gfx_backend::Resources, <ShaderFormat as gfx::format::Formatted>::Surface>,
    pub srv: gfx::handle::ShaderResourceView<gfx_backend::Resources, <ShaderFormat as gfx::format::Formatted>::View>,
    pub sampler: gfx::handle::Sampler<gfx_backend::Resources>, _phantom: PhantomData<P>
}

impl<P: Pipeline> Texture<P> {
    pub fn new(
        factory: &mut gfx_backend::Factory,
        image: &DynamicImage
    ) -> Result<Self, RenderError> {
        let (tex, srv) = factory.create_texture_immutable_u8::<ShaderFormat>(
            gfx::texture::Kind::D2(
                image.width() as u16,
                image.height() as u16,

                gfx::texture::AaMode::Single
            ),

            gfx::texture::Mipmap::Provided,

            &[&image.to_rgba().into_raw()]
        )
            .map_err(|err| RenderError::CombinedError(err))?;

        Ok(Self {
            tex,
            srv,

            sampler: factory.create_sampler(gfx::texture::SamplerInfo::new(
                gfx::texture::FilterMethod::Scale,
                gfx::texture::WrapMode::Clamp
            )),

            _phantom: PhantomData
        })
    }

	pub fn new_dynamic(
		factory: &mut gfx_backend::Factory,

		width: u16,
		height: u16
	) -> Result<Self, RenderError> {
		let tex = factory.create_texture(
			gfx::texture::Kind::D2(
				width,
				height,

				gfx::texture::AaMode::Single
			),

			1 as gfx::texture::Level,

			gfx::memory::Bind::SHADER_RESOURCE,
			gfx::memory::Usage::Dynamic,

			Some(<<ShaderFormat as gfx::format::Formatted>::Channel as gfx::format::ChannelTyped>::get_channel_type())
		)
			.map_err(|err| RenderError::CombinedError(gfx::CombinedError::Texture(err)))?;

		let srv = factory.view_texture_as_shader_resource::<ShaderFormat>(&tex, (0, 0), gfx::format::Swizzle::new())
			.map_err(|err| RenderError::CombinedError(gfx::CombinedError::Resource(err)))?;

		Ok(Self {
            tex,
            srv,
			
            // todo: é esse o sampler correto?
            sampler: factory.create_sampler(gfx::texture::SamplerInfo::new(
                gfx::texture::FilterMethod::Scale,
				
                // gfx::texture::FilterMethod::Bilinear
                gfx::texture::WrapMode::Clamp
            )),
			
            _phantom: PhantomData
        })
	}

	// atualizar textura com o dado fornecido (utilizado para atualizar a textura glyphcache)
	pub fn update(
		&self,

		encoder: &mut gfx::Encoder<gfx_backend::Resources, gfx_backend::CommandBuffer>,

		offset: [u16; 2],
        size: [u16; 2],
        data: &[[u8; 4]]
	) -> Result<(), RenderError> {
		let info = gfx::texture::ImageInfoCommon {
            xoffset: offset[0],
            yoffset: offset[1],
            zoffset: 0,
			
            width: size[0],
            height: size[1],
			
            depth: 0,
            format: (),
            mipmap: 0
        };

		encoder
			.update_texture::<<ShaderFormat as gfx::format::Formatted>::Surface, ShaderFormat>(&self.tex, None, info, data)
			.map_err(|err| RenderError::TexUpdateError(err))
	}
}
