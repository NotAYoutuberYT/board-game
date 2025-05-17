use rand::seq::SliceRandom;

use super::villager::{DeadVillager, LivingVillager, Villager, VillagerType};

struct Village {
    living_villagers: Vec<LivingVillager>,
    dead_villagers: Vec<DeadVillager>,
}

impl Village {
    pub fn new(normal_villagers: usize, murderers: usize) -> Self {
        let mut rng = rand::rng();
        let mut ids: Vec<usize> = (1..=(normal_villagers + murderers)).collect();
        ids.shuffle(&mut rng);

        let normal_ids = ids.iter().take(normal_villagers);
        let normal_villagers = normal_ids
            .into_iter()
            .map(|i| Villager::new(VillagerType::Normal, *i));

        let murderer_ids = ids.iter().take(murderers);
        let murderers = murderer_ids
            .into_iter()
            .map(|i| Villager::new(VillagerType::Murderer, *i));

        let mut villagers: Vec<LivingVillager> = Vec::new();
        villagers.extend(normal_villagers);
        villagers.extend(murderers);

        Self {
            living_villagers: villagers,
            dead_villagers: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::village::villager::VillagerType;

    use super::Village;

    #[test]
    fn test_correct_villager_count() {
        let village = Village::new(4, 2);
        assert_eq!(village.living_villagers.len(), 6);
        assert_eq!(
            village
                .living_villagers
                .iter()
                .filter(|villager| villager.kind() == &VillagerType::Normal)
                .count(),
            4
        );
        assert_eq!(
            village
                .living_villagers
                .iter()
                .filter(|villager| villager.kind() == &VillagerType::Murderer)
                .count(),
            2
        );
    }
}
