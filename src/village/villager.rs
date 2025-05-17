use std::marker::PhantomData;

#[derive(PartialEq, Eq)]
pub enum VillagerType {
    Normal,
    Murderer,
}

pub enum Alive {}
pub enum Dead {}

pub trait VillagerStatus {}
impl VillagerStatus for Alive {}
impl VillagerStatus for Dead {}

pub type LivingVillager = Villager<Alive>;
pub type DeadVillager = Villager<Dead>;

pub struct Villager<S: VillagerStatus> {
    kind: VillagerType,
    label: usize,
    marker: PhantomData<S>,
}

impl LivingVillager {
    pub fn new(kind: VillagerType, label: usize) -> Self {
        Self {
            kind,
            label,
            marker: PhantomData,
        }
    }

    pub fn kind(&self) -> &VillagerType {
        &self.kind
    }

    pub fn kill(self) -> Villager<Dead> {
        Villager {
            kind: self.kind,
            label: self.label,
            marker: PhantomData,
        }
    }
}
