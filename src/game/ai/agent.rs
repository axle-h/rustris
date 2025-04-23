use crate::game::ai::apply_inputs::ApplyInputs;
use crate::game::ai::board_cost::BoardCost;
use crate::game::ai::input_search::InputSearch;
use crate::game::ai::input_sequence::InputSequence;
use crate::game::{Game, GameState};
use crate::game::tetromino::TetrominoShape;

pub struct AiAgent {
    cost: BoardCost
}

impl AiAgent {

    pub fn new(cost: BoardCost) -> Self {
        Self { cost }
    }

    pub fn act(&self, game: &mut Game) -> bool {
        if !matches!(game.state, GameState::Fall(_)) {
            return false;
        }
        
        if let Some(shape) = game.board.tetromino().map(|t| t.shape()) {
            let best_result = self.best_move(game, shape);

            // let alt_next_shape = game.hold.map(|state| state.shape).unwrap_or_else(|| game.random.peek());
            // let alt_best_move = self.best_move(game, alt_next_shape);
            
            // let (best_inputs, is_alt) = match (best_result, alt_best_move) {
            //     (None, None) => return false,
            //     (Some((m, _)), None) => (m, false),
            //     (None, Some((m, _))) => (m, true),
            //     (Some((m1, c1)), Some((m2, c2))) =>
            //         if c1 < c2 { (m1, false) } else { (m2, true) }
            // };
            // 
            
            let is_alt = false;
            if best_result.is_none() {
                return false;
            }
            let (best_inputs, _) = best_result.unwrap();


            println!("{:?}", best_inputs);
            
            if is_alt {
                // return and wait for a tetromino
                return game.hold()
            }

            if !game.apply_inputs(best_inputs) {
                panic!("cannot apply inputs")
            }
            if !game.hard_drop() {
                panic!("cannot hard drop")
            }
            true
        } else {
            false
        }
    }
    
    fn best_move(&self, game: &Game, shape: TetrominoShape) -> Option<(InputSequence, f32)> {
        game.board.search_all_inputs(shape)
            .into_iter()
            .map(|r| (r.inputs(), self.cost.cost(r.board())))
            .min_by(|(_, c1), (_, c2)| c1.total_cmp(&c2))
    }
}