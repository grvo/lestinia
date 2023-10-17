use std::{
	collections::HashMap,
	ops::Range,

	u64,
	
	fmt
};

use specs::{
	saveload::{
		Marker,
		MarkerAllocator
	},

	world::EntitiesRes,

	Component,
	VecStorage,
	Entity,
	Join,
	ReadStorage
};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Uid(pub u64);

impl Into<u64> for Uid {
	fn into(self) -> u64 {
		self.0
	}
}

impl fmt::Display for Uid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl Component for Uid {
	type Storage = VecStorage<Self>;
}

impl Marker for Uid {
	type Identifier = u64;
	type Allocator = UidAllocator;

	fn id(&self) -> u64 {
		self.0
	}

	fn update(&mut self, update: Self) {
		assert_eq!(self.0, update.0);
	}
}

pub struct UidAllocator {
	pub(crate) range: Range<u64>,
	pub(crate) mapping: HashMap<u64, Entity>
}

impl UidAllocator {
	pub fn new() -> Self {
		Self {
			range: 0..64::MAX,
			mapping: HashMap::new()
		}
	}
}

impl MarkerAllocator<Uid> for UidAllocator {
	fn allocate(&mut self, entity: Entity, id: Option<u64>) -> Uid {
		let id = id.unwrap_or_else(|| {
			self.range.next().expect("
   				alcance de id deve ser efetivamente interminável.
	   			de alguma maneira, você pode rodar esse programa por mais tempo do que o tempo de vida do universo.
	   			é provavelmente a hora de parar de jogar e preparar uma extinção eminente.
   			")
		});

		self.mapping.insert(id, entity);

		Uid(id)
	}

	fn retrieve_entity_internal(&self, id: u64) -> Option<Entity> {
		self.mapping.get(&id).cloned()
	}

	fn maintain(&mut self, entities: &EntitiesRes, storage: &ReadStorage<Uid>) {
		self.mapping = (&*entities, storage)
			.join()
			.map(|(e, m)| (m.id(), e))
			.collect();
	}
}
