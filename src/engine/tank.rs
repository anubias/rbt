use crate::{api::player::Player, engine::context::Context};

pub struct Tank {
    context: Context,
    player: Box<dyn Player>,
}

impl Tank {
    pub fn new(player: Box<dyn Player>, context: Context) -> Self {
        Self { context, player }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn player(&self) -> &Box<dyn Player> {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Box<dyn Player> {
        &mut self.player
    }

    pub fn set_context(&mut self, context: Context) {
        self.context = context;
    }

    pub fn health_bar(&self) -> String {
        const BAR_UNIT: u8 = 20;

        let length = self.context.health() / BAR_UNIT;
        let mut bar = String::new();

        for i in 0..100 / BAR_UNIT {
            let char = if i < length { '=' } else { ' ' };
            bar = format!("{}{}", bar, char);
        }

        bar
    }

    pub fn survivor_bonus(&mut self) {
        self.context.reward_survivor();
    }
}
