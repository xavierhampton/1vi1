pub enum GameState {
    RoundStart { timer: f32 },
    Playing,
    RoundEnd { winner_name: String, winner_color: (u8, u8, u8), timer: f32 },
}
