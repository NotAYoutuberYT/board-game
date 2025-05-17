use crate::uni::Uni;

pub struct RealPlayer {
    owned_unis: Vec<Uni>,
}

impl RealPlayer {
    pub fn new() -> Player {
        Player::Real(Self { owned_unis: Vec::new()})
    }

    pub fn give_uni(&mut self, uni: Uni) {
        self.owned_unis.push(uni);
    }
}

pub struct ComputerPlayer;

impl ComputerPlayer {
    pub fn new() -> Player {
        Player::Computer(Self)
    }
}

pub enum Player {
    Real(RealPlayer),
    Computer(ComputerPlayer)
}