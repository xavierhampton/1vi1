pub enum GameState {
    RoundStart { timer: f32 },
    Playing,
    RoundEnd { winner_index: u8, winner_name: String, winner_color: (u8, u8, u8), timer: f32 },
    CardPick {
        winner_index: u8,
        current_picker: u8,       // player index currently choosing
        offered_cards: [u8; 3],   // 3 card IDs from catalog
        pick_order: Vec<u8>,      // remaining losers to pick (player indices)
        phase_timer: f32,         // entrance delay
        chosen_card: Option<u8>,  // 0-2 once picked (card slot index, not card ID)
        exit_timer: f32,          // post-pick animation
    },
    MatchOver { winner_index: u8, timer: f32 },
}
