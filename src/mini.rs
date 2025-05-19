use crate::village::{Village, VillagerType};

/// an action a mini can take
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    PostRegister,
    PostFlare,
    Detonate,
    Visit,
}

/// an operation on a mini's register
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operation {
    Increment,
    Decrement,
    SetValue(u8),
}

/// a conditional
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Condition {
    VillagerIsAlive,
    VillagerIsDead,
    RegisterEq(u8),
}

/// any instruction a mini can run
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Instruction {
    Action(Action),
    Operation(Operation),
    Condition(Condition, Instructions),
    /// for infinite loop protection, decrement u8 each iteration; if it hits zero, break.
    Repeat(u8, Instructions),
    Break,
}

pub type Instructions = Vec<Instruction>;

/// something that can be posted to a mini's log
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Event {
    PostedRegister(u8),
    PostedFlare,
    Finished,
}

pub type EventLog = Vec<Event>;

/// the status of a mini: running, or various ways for it to stop working
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MiniStatus {
    Running,
    Done,
    Destroyed,
    Lost,
}

/// a mini, along with all the information it needs to run:
/// an instruction stack register, log, etc.
pub struct Mini {
    /// because this is a stack, the "next" instruction is at the end of the vector
    instruction_stack: Instructions,
    register: u8,

    status: MiniStatus,
    location: u8,
    log: EventLog,
}

impl Mini {
    /// construct a new mini. requires a reference to a village so that the mini can
    /// visit its starting location
    pub fn new(starting_location: u8, base_instructions: Instructions, village: &Village) -> Self {
        let mut mini = Self {
            instruction_stack: base_instructions,
            register: 0,
            status: MiniStatus::Running,
            location: starting_location,
            log: Vec::new(),
        };

        mini.visit_villager(village, starting_location);

        mini
    }

    pub fn log(&self) -> &EventLog {
        &self.log
    }

    /// updates location (or becomes lost) and then carries out the
    /// appropriate action according to the type of the visited villager
    fn visit_villager(&mut self, village: &Village, location: u8) {
        if !village.villager_exists(location) {
            self.status = MiniStatus::Lost;
            return;
        }

        self.location = location;

        // if the villager is dead, we shouldn't do anything
        if village.dead_villager(location).is_some() {
            return;
        }

        // do whatever the visited villager's type entails
        match village
            .villager_type(location)
            .expect("just confirmed villager exists")
        {
            VillagerType::Murderer => {
                self.status = MiniStatus::Destroyed;
                self.log = Vec::new();
            }
            VillagerType::Afraid => self.status = MiniStatus::Destroyed,
            _ => (),
        }
    }

    /// pop the top instruction off the instruction stack and run it
    fn run_instruction(&mut self, village: &mut Village) {
        // get the next instruction. if there are no more instructions, set
        // our status to done
        let instruction = match self.instruction_stack.pop() {
            Some(instruction) => instruction,
            None => {
                self.status = MiniStatus::Done;
                return;
            }
        };

        // match the instruction against every possible value and
        // do whatever is required by the instruction
        match instruction {
            Instruction::Action(Action::PostRegister) => {
                self.log.push(Event::PostedRegister(self.register))
            }
            Instruction::Action(Action::PostFlare) => self.log.push(Event::PostedFlare),
            Instruction::Action(Action::Detonate) => {
                let _ = village.kill_villager(self.register);
                self.status = MiniStatus::Destroyed;
            }
            Instruction::Action(Action::Visit) => self.visit_villager(village, self.register),

            Instruction::Operation(Operation::Increment) => {
                // destroy the mini if we'd encounter overflow
                if self.register == u8::MAX {
                    self.status = MiniStatus::Destroyed
                } else {
                    self.register += 1
                }
            }
            Instruction::Operation(Operation::Decrement) => {
                // destroy the mini if we'd encounter underflow
                if self.register == 0 {
                    self.status = MiniStatus::Destroyed
                } else {
                    self.register -= 1;
                }
            }
            Instruction::Operation(Operation::SetValue(value)) => self.register = value,

            Instruction::Condition(Condition::VillagerIsAlive, instructions) => {
                // if the villager we're at is alive, push the conditional
                // instructions to the stack
                if village.living_villager(self.location).is_some() {
                    self.instruction_stack.extend(instructions);
                }
            }
            Instruction::Condition(Condition::VillagerIsDead, instructions) => {
                // if the villager we're at is dead, push the conditional
                // instructions to the stack
                if village.dead_villager(self.location).is_some() {
                    self.instruction_stack.extend(instructions);
                }
            }
            Instruction::Condition(Condition::RegisterEq(value), instructions) => {
                // if register is equal to the test value, push the conditional instructions to the stack
                if self.register == value {
                    self.instruction_stack.extend(instructions);
                }
            }

            Instruction::Repeat(iterations, instructions) => {
                // if we're not out of iterations (i.e. we aren't facing an infinite loop),
                // put the repeated instructions to the stack along with a copy of this
                // repeat command with the remaining iteration count decreased by one
                if iterations != 0 {
                    self.instruction_stack
                        .push(Instruction::Repeat(iterations - 1, instructions.clone()));
                    self.instruction_stack.extend(instructions);
                }
            }

            Instruction::Break => loop {
                // keep removing instructions from the stack until we've removed everything
                // or encountered and removed a repeat instruction (which will end up being
                // the most nested repeat)
                match self.instruction_stack.pop() {
                    None => {
                        self.status = MiniStatus::Done;
                        break;
                    }
                    Some(Instruction::Repeat(_, _)) => break,
                    Some(_) => (),
                }
            },
        }
    }

    /// keep running instructions on the instruction stack until
    /// the state changes from running. the first instruction
    /// should be visit.
    pub fn run_until_completion(&mut self, village: &mut Village) {
        while self.status == MiniStatus::Running {
            self.run_instruction(village);
        }

        // if we finished gracefully (i.e. weren't destroyed or anything,
        // push the finish event to the log)
        if self.status == MiniStatus::Done {
            self.log.push(Event::Finished);
        }
    }
}

#[cfg(test)]
mod test {
    // recall in all of these tests that the instruction
    // stack is read back to front
    use std::vec;

    use crate::{
        mini::{Event, MiniStatus},
        village::{LivingVillager, Village, Villager, VillagerType},
    };

    use super::{Action, Condition, Instruction, Mini, Operation};

    #[test]
    fn register_operations() {
        let mut village = Village::new_deterministic(vec![Villager::new(VillagerType::Normal, 1)]);

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Operation(Operation::Decrement),
                Instruction::Operation(Operation::SetValue(10)),
                Instruction::Operation(Operation::Decrement),
                Instruction::Operation(Operation::Increment),
                Instruction::Operation(Operation::Increment),
            ],
            &village,
        );

        assert_eq!(mini.register, 0);
        mini.run_instruction(&mut village);
        assert_eq!(mini.register, 1);
        mini.run_instruction(&mut village);
        assert_eq!(mini.register, 2);
        mini.run_instruction(&mut village);
        assert_eq!(mini.register, 1);
        mini.run_instruction(&mut village);
        assert_eq!(mini.register, 10);
        mini.run_instruction(&mut village);
        assert_eq!(mini.register, 9);

        assert_eq!(mini.status, MiniStatus::Running);
    }

    #[test]
    fn register_safety() {
        let mut village = Village::new_deterministic(Vec::new());

        let mut mini = Mini::new(
            0,
            vec![Instruction::Operation(Operation::Decrement)],
            &village,
        );

        assert_eq!(mini.register, 0);
        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Destroyed);
        assert_eq!(mini.register, 0);

        let mut mini = Mini::new(
            0,
            vec![
                Instruction::Operation(Operation::Increment),
                Instruction::Operation(Operation::SetValue(u8::MAX)),
            ],
            &village,
        );

        mini.run_instruction(&mut village);
        assert_eq!(mini.register, u8::MAX);
        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Destroyed);
        assert_eq!(mini.register, u8::MAX);
    }

    #[test]
    fn visiting() {
        let villagers: Vec<LivingVillager> = (1..=4)
            .map(|i| Villager::new(VillagerType::Normal, i))
            .collect();
        let mut village = Village::new_deterministic(villagers);

        let mut mini = Mini::new(
            4,
            vec![
                Instruction::Action(Action::Visit),
                Instruction::Operation(Operation::Increment),
                Instruction::Action(Action::Visit),
                Instruction::Operation(Operation::SetValue(2)),
            ],
            &village,
        );

        assert_eq!(mini.location, 4);
        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        assert_eq!(mini.location, 2);
        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        assert_eq!(mini.location, 3);

        assert_eq!(mini.status, MiniStatus::Running);
        (1..=4).for_each(|i| assert!(village.living_villager(i).is_some()));
    }

    #[test]
    fn actions() {
        let villagers: Vec<LivingVillager> = (1..=4)
            .map(|i| Villager::new(VillagerType::Normal, i))
            .collect();
        let mut village = Village::new_deterministic(villagers);

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Action(Action::Detonate),
                Instruction::Action(Action::Visit),
                Instruction::Action(Action::PostRegister),
                Instruction::Action(Action::PostFlare),
                Instruction::Operation(Operation::SetValue(2)),
                Instruction::Action(Action::PostRegister),
            ],
            &village,
        );

        mini.run_instruction(&mut village);
        assert_eq!(mini.log, vec![Event::PostedRegister(0)]);
        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        assert_eq!(
            mini.log,
            vec![
                Event::PostedRegister(0),
                Event::PostedFlare,
                Event::PostedRegister(2)
            ]
        );

        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Running);
        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Destroyed);
        assert!(village.dead_villager(2).is_some());
    }

    #[test]
    fn dies_to_murderer() {
        let mut villagers: Vec<LivingVillager> = (1..=4)
            .map(|i| Villager::new(VillagerType::Normal, i))
            .collect();
        villagers.push(Villager::new(VillagerType::Murderer, 5));
        let mut village = Village::new_deterministic(villagers);

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Action(Action::Visit),
                Instruction::Operation(Operation::SetValue(5)),
            ],
            &village,
        );

        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Destroyed);
    }

    #[test]
    fn completes_correctly() {
        let villagers: Vec<LivingVillager> = (1..=4)
            .map(|i| Villager::new(VillagerType::Normal, i))
            .collect();
        let mut village = Village::new_deterministic(villagers);

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Operation(Operation::Increment),
                Instruction::Break,
            ],
            &village,
        );

        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Done);

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Operation(Operation::Increment),
                Instruction::Operation(Operation::Increment),
            ],
            &village,
        );

        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        mini.run_instruction(&mut village);
        assert_eq!(mini.status, MiniStatus::Done);
    }

    #[test]
    fn conditionals() {
        let villagers: Vec<LivingVillager> = vec![
            Villager::new(VillagerType::Normal, 1),
            Villager::new(VillagerType::Normal, 2),
        ];
        let mut village = Village::new_deterministic(villagers);
        village
            .kill_villager(2)
            .expect("we have a villager with id 2");

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Condition(
                    Condition::VillagerIsDead,
                    vec![Instruction::Action(Action::PostRegister)],
                ),
                Instruction::Action(Action::Visit),
                Instruction::Operation(Operation::SetValue(2)),
                Instruction::Condition(
                    Condition::VillagerIsAlive,
                    vec![Instruction::Action(Action::PostRegister)],
                ),
            ],
            &village,
        );

        mini.run_until_completion(&mut village);
        assert_eq!(
            mini.log,
            vec![
                Event::PostedRegister(0),
                Event::PostedRegister(2),
                Event::Finished
            ]
        );

        let mut mini = Mini::new(
            1,
            vec![
                Instruction::Condition(
                    Condition::VillagerIsAlive,
                    vec![Instruction::Action(Action::PostRegister)],
                ),
                Instruction::Action(Action::Visit),
                Instruction::Operation(Operation::SetValue(2)),
                Instruction::Condition(
                    Condition::VillagerIsDead,
                    vec![Instruction::Action(Action::PostRegister)],
                ),
            ],
            &village,
        );

        mini.run_until_completion(&mut village);
        assert_eq!(mini.log, vec![Event::Finished]);
    }

    #[test]
    fn repeat() {
        let mut village = Village::new_deterministic(vec![Villager::new(VillagerType::Normal, 1)]);

        // keep posting the register until it's equal to 4
        let mut mini = Mini::new(
            1,
            vec![Instruction::Repeat(
                u8::MAX,
                vec![
                    Instruction::Operation(Operation::Increment),
                    Instruction::Action(Action::PostRegister),
                    Instruction::Condition(Condition::RegisterEq(10), vec![Instruction::Break]),
                ],
            )],
            &village,
        );

        mini.run_until_completion(&mut village);

        // this also ensures break clears the rest of the active loop; if it didn't, 10 would be posted
        let mut events: Vec<Event> = (0..=9).map(|i| Event::PostedRegister(i)).collect();
        events.push(Event::Finished);
        assert_eq!(mini.log, events);
    }

    #[test]
    fn infinite_loop() {
        let mut village = Village::new_deterministic(vec![Villager::new(VillagerType::Normal, 1)]);

        // keep posting the register until it's equal to 4
        let mut mini = Mini::new(
            1,
            vec![Instruction::Repeat(
                10, // this should usually be u8::Max, but this makes the test faster
                vec![],
            )],
            &village,
        );

        mini.run_until_completion(&mut village);

        // this also ensures break clears the rest of the active loop; if it didn't, 10 would be posted
        assert!(mini.register < u8::MAX)
    }
}
