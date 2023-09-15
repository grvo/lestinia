pub mod cell;

// biblioteca
use vek::*;
use dot_vox::DotVoxData;

// caixote
use crate::{
    vol::{
        Vox,
        
        WriteVol
    },
    
    volumes::dyna::Dyna
};

// local
use self::cell::Cell;

/// um tipo representando uma figura única
#[derive(Copy, Clone)]
pub struct Bone {
    origin: Vec3<f32>,
    offset: Vec3<f32>,
    ori: Vec3<f32>
}

/// um tipo representando um volume que pode ser parte de uma figura animada
///
/// figuras são utilizadas para representar coisas como personagens, npcs, mobs, etc.
pub type Segment = Dyna<Cell, ()>;

impl From<DotVoxData> for Segment {
    fn from(dot_vox_data: DotVoxData) -> Self {
        if let Some(model) = dot_vox_data.models.get(0) {
            let palette = dot_vox_data
                .palette
                .iter()
                .map(|col| Rgba::from(col.to_ne_bytes()).into())
                .collect::<Vec<_>>();

            let mut segment = Segment::filled(
                Vec3::new(
                    model.size.x,
                    model.size.y,
                    model.size.z
                ),
                
                Cell::empty(),
                ()
            );

            for voxel in &model.voxels {
                if let Some(&color) = palette.get(voxel.i as usize) {
                    // TODO: talvez não ignorar esse erro?

                    let _ = segment.set(
                        Vec3::new(voxel.x, voxel.y, voxel.z).map(|e| e as i32),

                        Cell::new(color)
                    );
                }
            }

            segment
        } else {
            Segment::filled(Vec3::zero(), Cell::empty(), ())
        }
    }
}
