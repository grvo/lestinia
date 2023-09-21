// padrão
use std::marker::PhantomData;

// biblioteca
use vek::*;

// local
use crate::vol::{
    Vox,
    
    BaseVol,
    SizedVol,

    ReadVol,
    WriteVol,

    VolSize
};

#[derive(Debug)]
pub enum ChunkErr {
    OutOfBounds
}

/// um volume com dimensões conhecidas em tempo de compilação
// v = voxel
// s = tamanho (size)
// m = metadata de chunk
pub struct Chunk<V: Vox, S: VolSize, M> {
    vox: Vec<V>,
    meta: M,
    phantom: PhantomData<S>
}

impl<V: Vox, S: VolSize, M> Chunk<V, S, M> {
    /// utilizado para transformar a posição de voxel em um volume no index correspondente no array do voxel
    #[inline(always)]

    fn idx_for(pos: Vec3<i32>) -> Option<usize> {
        if
            pos.map(|e| e >= 0).reduce_and() &&
            pos.map2(S::SIZE, |e, lim| e < lim as i32).reduce_and()
        {
            Some((
                pos.x * S::SIZE.y as i32 * S::SIZE.z as i32 +
                pos.y * S::SIZE.z as i32 +
                pos.z
            ) as usize)
        } else {
            None
        }
    }
}

impl<V: Vox, S: VolSize, M> BaseVol for Chunk<V, S, M> {
    type Vox = V;

    type Err = ChunkErr;
}

impl<V: Vox, S: VolSize, M> SizedVol for Chunk<V, S, M> {
    #[inline(always)]
    fn get_size(&self) -> Vec3<u32> { S::SIZE }
}

impl<V: Vox, S: VolSize, M> ReadVol for Chunk<V, S, M> {
    #[inline(always)]
    
    fn get(&self, pos: Vec3<i32>) -> Result<&V, ChunkErr> {
        Self::idx_for(pos)
            .and_then(|idx| self.vox.get(idx))
            .ok_or(ChunkErr::OutOfBounds)
    }
}

impl<V: Vox, S: VolSize, M> WriteVol for Chunk<V, S, M> {
    #[inline(always)]

    fn set(&mut self, pos: Vec3<i32>, vox: Self::Vox) -> Result<(), ChunkErr> {
        Self::idx_for(pos)
            .and_then(|idx| self.vox.get_mut(idx))
            .map(|old_vox| *old_vox = vox)
            .ok_or(ChunkErr::OutOfBounds)
    }
}

impl<V: Vox: Clone, S: VolSize, M> Chunk<V, S, M> {
    /// cria um novo chunk com as dimensões fornecidas e todos os voxels alinhados com duplicáveis no voxel fornecido
    pub fn filled(vox: V, meta: M) -> Self {
        Self {
            vox: vec![vox; S::SIZE.product() as usize],
            meta,
            phantom: PhantomData
        }
    }

    /// obtém uma referência na metadata interna
    pub fn metadata(&self) -> &M {
        &self.meta
    }

    /// obtém uma referência mutável na metadata interna
    pub fn metadata_mut(&mut self) -> &mut M {
        &mut self.meta
    }
}
