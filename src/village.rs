use std::marker::PhantomData;

use rand::{
    random_bool,
    seq::SliceRandom,
};
use thiserror::Error;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VillagerType {
    Normal,
    /// strong villagers can survive one attack (if the bool is true, they haven't used their resistance yet)
    Strong(bool),
    /// afraid villagers kill minis (but won't delete their logs)
    Afraid,
    Murderer,
}

/// villagers have two states: Alive and Dead
#[derive(Clone, Copy)]
pub enum Alive {}
#[derive(Clone, Copy)]
pub enum Dead {}

// declare Alive and Dead as villager statuses
pub trait VillagerStatus {}
impl VillagerStatus for Alive {}
impl VillagerStatus for Dead {}

pub type LivingVillager = Villager<Alive>;
pub type DeadVillager = Villager<Dead>;

/// represents a villager. for the purpose of enforcing things along the
/// lines of not constructing a dead villager and not killing living
/// villagers and requiring living and dead villagers to be handled
/// separately, the villager type is tied to the Alive or Dead state.
#[derive(Clone, Copy)]
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

    /// used to update types that contain state
    pub fn set_kind(&mut self, kind: VillagerType) {
        self.kind = kind;
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
    pub fn label(&self) -> u8 {
        self.label
    }

    pub fn has_label(&self, label: u8) -> bool {
        self.label == label
    }

    pub fn kind(&self) -> VillagerType {
        self.kind
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VillageStatus {
    Running,
    VillagersWon,
    MurdersWon,
}

pub struct Village {
    living_villagers: Vec<LivingVillager>,
    dead_villagers: Vec<DeadVillager>,
    status: VillageStatus,

    /// the original layout of the village. shown
    /// to the user at the end of the game.
    layout: Vec<LivingVillager>,
}

impl Village {
    /// constructs a village with the specified number of various types of
    /// villagers. randomly generates the ordering/labeling of the villagers.
    pub fn new(
        normal_villagers: u8,
        strong_villagers: u8,
        afraid_villagers: u8,
        murderers: u8,
    ) -> Self {
        let normal_villagers =
            (0..normal_villagers).map(|_| Villager::new(VillagerType::Normal, 0));

        let strong_villagers =
            (0..strong_villagers).map(|_| Villager::new(VillagerType::Strong(true), 0));

        let afraid_villagers =
            (0..afraid_villagers).map(|_| Villager::new(VillagerType::Afraid, 0));

        let murderers = (0..murderers).map(|_| Villager::new(VillagerType::Murderer, 0));

        let mut villagers: Vec<LivingVillager> = Vec::new();
        villagers.extend(normal_villagers);
        villagers.extend(strong_villagers);
        villagers.extend(afraid_villagers);
        villagers.extend(murderers);

        let mut rng = rand::rng();
        let mut ids: Vec<usize> = (1..=villagers.len()).collect();
        ids.shuffle(&mut rng);

        villagers.iter_mut().enumerate().for_each(|(i, villager)| {
            villager.set_label(*ids.get(i).expect("we have enough ids") as u8)
        });

        Self {
            living_villagers: villagers.clone(),
            dead_villagers: Vec::new(),
            status: VillageStatus::Running,
            layout: villagers,
        }
    }

    /// for testing purposes. constructs a village with a pre-determined set of villagers
    #[allow(dead_code)]
    pub fn new_deterministic(villagers: Vec<LivingVillager>) -> Self {
        Self {
            living_villagers: villagers.clone(),
            dead_villagers: Vec::new(),
            status: VillageStatus::Running,
            layout: villagers,
        }
    }

    pub fn layout(&self) -> Vec<LivingVillager> {
        self.layout.clone()
    }

    pub fn status(&self) -> VillageStatus {
        self.status
    }

    /// checks if murders or villagers have won. updates status accordingly.
    pub fn update_status(&mut self) {
        let murderers = self
            .living_villagers
            .iter()
            .filter(|villager| villager.kind() == VillagerType::Murderer)
            .count();

        if murderers == 0 {
            self.status = VillageStatus::VillagersWon;
        } else if murderers == self.living_villagers.len() {
            self.status = VillageStatus::MurdersWon;
        }
    }

    /// have each murderer attempt to kill a villager and update the village's status
    pub fn run_night(&mut self) {
        // get the labels of all living murderers
        let murderers: Vec<u8> = self
            .living_villagers
            .iter()
            .filter_map(|villager| match villager.kind() {
                VillagerType::Murderer => Some(villager.label()),
                _ => None,
            })
            .collect();

        for murder_label in murderers {
            // get all possible labels of neighbors above and below this murderer.
            // these are ordered from closes to furthest away from the murderer.
            let mut neighbors_above = (murder_label + 1)..=u8::MAX;
            let mut neighbors_below = (1..murder_label).rev();

            // get the label of the nearest living villager above and below the murderer
            let to_kill_above = neighbors_above.find(|label| {
                self.living_villager(*label)
                    .map(|villager| villager.kind() != VillagerType::Murderer)
                    .unwrap_or(false)
            });
            let to_kill_below = neighbors_below.find(|label| {
                self.living_villager(*label)
                    .map(|villager| villager.kind() != VillagerType::Murderer)
                    .unwrap_or(false)
            });

            // randomly pick the villager above or below
            // (even if one is empty/None)
            let to_kill = match random_bool(0.5) {
                true => to_kill_above,
                false => to_kill_below,
            };

            // extract the actual label (right now we just have an option)
            let to_kill = match to_kill {
                Some(label) => label,
                None => continue,
            };

            // kill the villager (note the extra complexity to make sure we
            // properly handle strong villagers)
            match self
                .villager_type(to_kill)
                .expect("the label came from an existing villager")
            {
                VillagerType::Strong(true) => self
                    .living_villager_mut(to_kill)
                    .expect("the label came from an existing villager")
                    .set_kind(VillagerType::Strong(false)),
                _ => self
                    .kill_villager(to_kill)
                    .expect("the label came from an existing villager"),
            }
        }

        self.update_status();
    }

    /// checks if a certain villager exists dead or alive
    pub fn villager_exists(&self, label: u8) -> bool {
        self.dead_villagers
            .iter()
            .any(|villager| villager.has_label(label))
            || self
                .living_villagers
                .iter()
                .any(|villager| villager.has_label(label))
    }

    /// attempt to get the living villager with the provided label
    pub fn living_villager(&self, label: u8) -> Option<&LivingVillager> {
        self.living_villagers
            .iter()
            .find(|villager| villager.has_label(label))
    }

    /// used to mutate villagers, particularly updating villager types that contain state
    pub fn living_villager_mut(&mut self, label: u8) -> Option<&mut LivingVillager> {
        self.living_villagers
            .iter_mut()
            .find(|villager| villager.has_label(label))
    }

    /// attempt to get the dead villager with the provided label
    pub fn dead_villager(&self, label: u8) -> Option<&DeadVillager> {
        self.dead_villagers
            .iter()
            .find(|villager| villager.has_label(label))
    }

    /// attempt to get the type of the dead or alive villager with the provided label
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

    /// attempts to kill the villager with the provided label
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

/// represents anything that can go wrong with village operations.
/// is small right now, but could grow if more features are added.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum VillageError {
    /// the u8 represents the label which couldn't be found
    #[error("villager `{0}` in incorrect state or does not exist")]
    NoSuchVillager(u8),
}

#[cfg(test)]
mod test {
    use crate::village::{VillageError, VillagerType};

    use super::Village;

    #[test]
    fn correct_villagers_on_creation() {
        let village = Village::new(5, 4, 3, 2);
        (1..=9).for_each(|i| assert!(village.living_villager(i).is_some()));

        let mut normal_villagers = 0;
        let mut strong_villagers = 0;
        let mut afraid_villagers = 0;
        let mut murderers = 0;

        village
            .living_villagers
            .iter()
            .for_each(|villager| match villager.kind() {
                VillagerType::Normal => normal_villagers += 1,
                VillagerType::Strong(_) => strong_villagers += 1,
                VillagerType::Afraid => afraid_villagers += 1,
                VillagerType::Murderer => murderers += 1,
            });

        assert_eq!(normal_villagers, 5);
        assert_eq!(strong_villagers, 4);
        assert_eq!(afraid_villagers, 3);
        assert_eq!(murderers, 2);
    }

    #[test]
    fn gets_correct_villagers() {
        let mut village = Village::new(5, 0, 0, 3);
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
        let mut village = Village::new(3, 0, 0, 3);
        assert!(village.kill_villager(2).is_ok());
        assert!(village.kill_villager(4).is_ok());
        assert!(village.kill_villager(2).unwrap_err() == VillageError::NoSuchVillager(2))
    }
}
