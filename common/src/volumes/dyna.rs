// biblioteca
use vek::*;

// local
use crate::vol::{
    Vox,
    
    BaseVol,
    SizedVol,
    ReadVol,
    WriteVol
};

pub enum DynaErr {
    OutOfBounds
}

/// um volume com dimensões conhecidas apenas na criação de objeto
// v = voxel
// s = tamanho
// m = metadata
pub struct Dyna<V: Vox, M> {
    vox: Vec<V>,
    meta: M,
    sz: Vec3<u32>
}

impl<V: Vox, M> Dyna<V, M> {
    /// utilizado para transformar a posição do voxel no volume ao index correspondente do array do voxel
    #[inline(always)]
    fn idx_for(sz: Vec3<u32>, pos: Vec3<i32>) -> Option<usize> {
        if
            pos.map(|e| e >= 0).reduce_and() &&
            pos.map2(sz, |e, lim| e < lim as i32).reduce_and()
        {
            Some((
                pos.x * sz.y as i32 * sz.z as i32 +
                pos.y * sz.z as i32 +
                pos.z
            ) as usize)
        } else {
            None
        }
    }
}

impl<V: Vox, M> BaseVol for Dyna<V, M> {
    type Vox = V;
    
    type Err = DynaErr;
}

impl<V: Vox, M> SizedVol for Dyna<V, M> {
    #[inline(always)]
    fn get_size(&self) -> Vec3<u32> { self.sz }
}

impl<V: Vox, M> ReadVol for Dyna<V, M> {
    #[inline(always)]
    fn get(&self, pos: Vec3<i32>) -> Result<&V, DynaErr> {
        Self::idx_for(self.sz, pos)
            .and_then(|idx| self.vox.get(idx))
            .ok_or(DynaErr::OutOfBounds)
    }
}

impl<V: Vox, M> WriteVol for Dyna<V, M> {
    #[inline(always)]
    fn set(&mut self, pos: Vec3<i32>, vox: Self::Vox) -> Result<(), DynaErr> {
        Self::idx_for(self.sz, pos)
            .and_then(|idx| self.vox.get_mut(idx))
            .map(|old_vox| *old_vox = vox)
            .ok_or(DynaErr::OutOfBounds)
    }
}

impl<V: Vox + Clone, M> Dyna<V, M> {
    /// cria um novo dyna com as dimensões fornecidas e todos os voxels alinhados com duplicáveis do voxel fornecido
    pub fn filled(sz: Vec3<u32>, vox: V, meta: M) -> Self {
        Self {
            vox: vec![vox; sz.product() as usize],
            meta,
            sz
        }
    }

    /// obtém uma referência para a metadata interna
    pub fn metadata(&self) -> &M {
        &self.meta
    }

    /// obtém uma referência mutável para a metadata interna
    pub fn metadata_mut(&mut self) -> &mut M {
        &mut self.meta
    }
}
