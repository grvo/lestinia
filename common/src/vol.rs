// biblioteca
use vek::*;

/// um volume que contém dados de voxel
pub trait BaseVol {
    type Vox;
    type Err;
}

// tipos de utilidade

pub struct VoxPosIter {
    pos: Vec3<u32>,
    sz: Vec3<u32>
}

impl Iterator for VoxPosIter {
    type Item = Vec3<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut old_pos = self.pos;

        if old_pos.z == self.sz.z {
            old_pos.z = 0;
            old_pos.y += 1;

            if old_pos.y == self.sz.y {
                old_pos.y = 0;
                old_pos.x += 1;

                if old_pos.x == self.sz.x {
                    return None;
                }
            }
        }

        self.pos = old_pos + Vec3::unit_z();

        Some(old_pos.map(|e| e as i32))
    }
}

/// volume que possui tamanho finito
pub trait SizedVol: BaseVol {
    /// obtém o tamanho do volume
    #[inline(always)]
    fn get_size(&self) -> Vec3<u32>;

    /// todas as posições do voxel em potencial nesse volume
    fn iter_positions(&self) -> VoxPosIter {
        VoxPosIter {
            pos: Vec3::zero(),
            sz: self.get_size()
        }
    }
}

/// volume que proporciona acesso de leitura para dados de voxel
pub trait ReadVol: BaseVol {
    /// obtém a referência do voxel com a posição fornecida no volume
    #[inline(always)]

    fn get(&self, pos: Vec3<i32>) -> Result<&Self::Vox, Self::Err>;
}

/// volume que fornece acesso de escrita para dados de voxel
pub trait WriteVol: BaseVol {
    /// determina o voxel na posição fornecida no volume do valor fornecido
    #[inline(always)]

    fn set(&mut self, pos: Vec3<i32>, vox: Self::Vox) -> Result<(), Self::Err>;
}

// características úteis

/// utilizado para especificar o tamanho de tempo de compilação do volume
/// existe como um sub-título até que consts genéricos estejam implementados
pub trait VolSize {
    const SIZE: Vec3<u32>;
}
