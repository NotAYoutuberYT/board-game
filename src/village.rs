use std::marker::PhantomData;

use rand::seq::SliceRandom;
use thiserror::Error;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VillagerType {
    Normal,
    /// strong villagers can survive one attack (if the bool is true, they haven't used their resistance yet)
    Strong(bool),
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
    label: u8,
    marker: PhantomData<S>,
}

impl LivingVillager {
    pub fn new(kind: VillagerType, label: u8) -> Self {
        Self {
            kind,
            label,
            marker: PhantomData,
        }
    }

    /// only used for village generation. can mess things up if used anywhere else.
    pub fn set_label(&mut self, label: u8) {
        self.label = label;
    }

    pub fn kill(self) -> Villager<Dead> {
        Villager {
            kind: self.kind,
            label: self.label,
            marker: PhantomData,
        }
    }
}

impl<S> Villager<S>
where
    S: VillagerStatus,
{
    pub fn has_label(&self, label: u8) -> bool {
        self.label == label
    }

    pub fn kind(&self) -> VillagerType {
        self.kind
    }
}

pub struct Village {
    living_villagers: Vec<LivingVillager>,
    dead_villagers: Vec<DeadVillager>,
}

impl Village {
    pub fn new(normal_villagers: u8, strong_villagers: u8, murderers: u8) -> Self {
        let normal_villagers =
            (0..normal_villagers).map(|_| Villager::new(VillagerType::Normal, 0));

        let strong_villagers =
            (0..strong_villagers).map(|_| Villager::new(VillagerType::Strong(true), 0));

        let murderers = (0..murderers).map(|_| Villager::new(VillagerType::Murderer, 0));

        let mut villagers = Vec::new();
        villagers.extend(normal_villagers);
        villagers.extend(strong_villagers);
        villagers.extend(murderers);

        let mut rng = rand::rng();
        let mut ids: Vec<usize> = (1..=villagers.len()).collect();
        ids.shuffle(&mut rng);

        villagers.iter_mut().enumerate().for_each(|(i, villager)| {
            villager.set_label(*ids.get(i).expect("we have enough ids") as u8)
        });

        Self {
            living_villagers: villagers,
            dead_villagers: Vec::new(),
        }
    }

    pub fn new_deterministic(villagers: Vec<LivingVillager>) -> Self {
        Self {
            living_villagers: villagers,
            dead_villagers: Vec::new(),
        }
    }

    pub fn living_villager(&self, label: u8) -> Option<&LivingVillager> {
        self.living_villagers
            .iter()
            .find(|villager| villager.has_label(label))
    }

    pub fn dead_villager(&self, label: u8) -> Option<&DeadVillager> {
        self.dead_villagers
            .iter()
            .find(|villager| villager.has_label(label))
    }

    pub fn villager_exists(&self, label: u8) -> bool {
        self.dead_villagers
            .iter()
            .find(|villager| villager.has_label(label))
            .is_some()
            || self
                .living_villagers
                .iter()
                .find(|villager| villager.has_label(label))
                .is_some()
    }

    pub fn villager_type(&self, label: u8) -> Result<VillagerType, VillageError> {
        let kind;
        if let Some(villager) = self.living_villager(label) {
            kind = villager.kind();
        } else {
            kind = self
                .dead_villager(label)
                .ok_or(VillageError::NoSuchVillager(label))?
                .kind();
        }

        Ok(kind)
    }

    pub fn kill_villager(&mut self, label: u8) -> Result<(), VillageError> {
        let position = self
            .living_villagers
            .iter()
            .position(|villager| villager.has_label(label))
            .ok_or(VillageError::NoSuchVillager(label))?;
        let villager = self.living_villagers.remove(position);
        self.dead_villagers.push(villager.kill());
        Ok(())
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VillageError {
    #[error("villager `{0}` in incorrect state or does not exist")]
    NoSuchVillager(u8),
}

#[cfg(test)]
mod test {
    use crate::village::{VillageError, VillagerType};

    use super::Village;

    #[test]
    fn correct_villagers_on_creation() {
        let village = Village::new(4, 3, 2);
        (1..=9).for_each(|i| assert!(village.living_villager(i).is_some()));

        let mut normal_villagers = 0;
        let mut strong_villagers = 0;
        let mut murderers = 0;

        village
            .living_villagers
            .iter()
            .for_each(|villager| match villager.kind() {
                VillagerType::Normal => normal_villagers += 1,
                VillagerType::Strong(_) => strong_villagers += 1,
                VillagerType::Murderer => murderers += 1,
            });

        assert_eq!(normal_villagers, 4);
        assert_eq!(strong_villagers, 3);
        assert_eq!(murderers, 2);
    }

    #[test]
    fn gets_correct_villagers() {
        let mut village = Village::new(5, 0, 3);
        village.kill_villager(2).unwrap();
        village.kill_villager(5).unwrap();

        for label in 1..=8 {
            match label {
                2 | 5 => {
                    assert!(village.living_villager(label).is_none());
                    assert!(village.dead_villager(label).is_some());
                }
                _ => {
                    assert!(village.living_villager(label).is_some());
                    assert!(village.dead_villager(label).is_none());
                }
            }

            assert!(village.villager_exists(label))
        }

        assert!(!village.villager_exists(9));
        assert!(!village.villager_exists(10));
    }

    #[test]
    fn cannot_kill_villager_twice() {
        let mut village = Village::new(3, 0, 3);
        assert!(village.kill_villager(2).is_ok());
        assert!(village.kill_villager(4).is_ok());
        assert!(village.kill_villager(2).unwrap_err() == VillageError::NoSuchVillager(2))
    }
}
