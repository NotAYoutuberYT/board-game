use crate::player::{ComputerPlayer, Player, RealPlayer};

pub enum GameState {
    Day,
    Night
}

pub struct Game {
    state: GameState,
    players: Vec<Player>,
}

impl Game {
    pub fn new(player_count: usize) -> Self {
        let mut players: Vec<Player> = Vec::with_capacity(player_count);
        players.push(RealPlayer::new());
        (1..player_count).for_each(|_| players.push(ComputerPlayer::new()));

        Self {
            state: GameState::Day,
            players,
        }
    }

    pub fn advance(&mut self) {
        match self.state {
            GameState::Day => (),
            GameState::Night => ()
        };

        self.state = match self.state {
            GameState::Day => GameState::Night,
            GameState::Night => GameState::Day
        };
    }
}
