// padrão
use std::collections::HashMap;

// biblioteca
use vek::*;

// caixote
use crate::{
    vol::{
        Vox,
        
        BaseVol,
        SizedVol,
        ReadVol,
        SampleVol,
        WriteVol,
        VolSize
    },

    volumes::{
        chunk::{
            Chunk,
            ChunkErr
        },

        dyna::{
            Dyna,
            DynaErr
        }
    }
};

pub enum VolMapErr {
    NoSuchChunk,

    ChunkErr(ChunkErr),
    DynaErr(DynaErr)
}

// v = voxel
// s = tamanho (size)
// m = metadata de chunk
pub struct VolMap<V: Vox, S: VolSize, M> {
    chunks: HashMap<Vec3<i32>, Chunk<V, S, M>>
}

impl<V: Vox, S: VolSize, M> VolMap<V, S, M> {
    #[inline(always)]

    fn chunk_key(pos: Vec3<i32>) -> Vec3<i32> {
        pos.map2(S::SIZE, |e, sz| e.div_euclid(sz as i32))
    }

    #[inline(always)]

    fn chunk_offs(pos: Vec3<i32>) -> Vec3<i32> {
        pos.map2(S::SIZE, |e, sz| e.rem_euclid(sz as i32))
    }
}

impl<V: Vox, S: VolSize, M> BaseVol for VolMap<V, S, M> {
    type Vox = V;
    
    type Err = VolMapErr;
}

impl<V: Vox, S: VolSize, M> ReadVol for VolMap<V, S, M> {
    #[inline(always)]

    fn get(&self, pos: Vec3<i32>) -> Result<&V, VolMapErr> {
        let ck = Self::chunk_key(pos);

        self.chunks.get(&ck)
            .ok_or(VolMapErr::NoSuchChunk)

            .and_then(|chunk| {
                let co = Self::chunk_offs(pos);
                
                chunk.get(co).map_err(|err| VolMapErr::ChunkErr(err))
            })
    }
}

impl<V: Vox + Clone, S: VolSize, M> SampleVol for VolMap<V, S, M> {
    type Sample = Dyna<V, ()>;

    /// obtém um sample do terreno por clonagem de voxels sem o alcance fornecido
    ///
    /// note que o volume resultante não possui metadata dos chunks originais
    fn sample(&self, range: Aabb<i32>) -> Result<Self::Sample, VolMapErr> {
        let mut sample = Dyna::filled(
            range.size().map(|e| e as u32).into(),
            V::empty(),
            ()
        );

        for pos in sample.iter_positions() {
            sample.set(pos, self.get(range.min + pos)?.clone())
                .map_err(|err| VolMapErr::DynaErr(err))?;
        }

        Ok(sample)
    }
}

impl<V: Vox, S: VolSize, M> WriteVol for VolMap<V, S, M> {    
    #[inline(always)]

    fn set(&mut self, pos: Vec3<i32>, vox: V) -> Result<(), VolMapErr> {
        let ck = Self::chunk_key(pos);

        self.chunks.get_mut(&ck)
            .ok_or(VolMapErr::NoSuchChunk)

            .and_then(|chunk| {
                let co = Self::chunk_offs(pos);

                chunk.set(co, vox).map_err(|err| VolMapErr::ChunkErr(err))
            })
    }
}

impl<V: Vox, S: VolSize, M> VolMap<V, S, M> {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}
