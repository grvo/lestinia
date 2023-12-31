pub mod run;

// re-exportações
pub use self::run::RunAnimation;

// caixote
use crate::render::FigureBoneData;

// local
use super::{
    Skeleton,
    Bone
};

pub struct CharacterSkeleton {
    head: Bone,
    chest: Bone,

    belt: Bone,
    shorts: Bone,

    l_hand: Bone,
    r_hand: Bone,
    
    l_foot: Bone,
    r_foot: Bone,

    back: Bone
}

impl CharacterSkeleton {
    pub fn new() -> Self {
        Self {
            head: Bone::default(),
            chest: Bone::default(),

            belt: Bone::default(),
            shorts: Bone::default(),
            
            l_hand: Bone::default(),
            r_hand: Bone::default(),
            
            l_foot: Bone::default(),
            r_foot: Bone::default(),

            back: Bone::default()
        }
    }
}

impl Skeleton for CharacterSkeleton {
    fn compute_matrices(&self) -> [FigureBoneData; 16] {
        let chest_mat = self.chest.compute_base_matrix();

        [
            FigureBoneData::new(self.head.compute_base_matrix()),
            FigureBoneData::new(chest_mat),

            FigureBoneData::new(self.belt.compute_base_matrix()),
            FigureBoneData::new(self.shorts.compute_base_matrix()),

            FigureBoneData::new(self.l_hand.compute_base_matrix()),
            FigureBoneData::new(self.r_hand.compute_base_matrix()),
            
            FigureBoneData::new(self.l_foot.compute_base_matrix()),
            FigureBoneData::new(self.r_foot.compute_base_matrix()),
            
            FigureBoneData::new(chest_mat * self.back.compute_base_matrix()),

            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default()
        ]
    }
}
