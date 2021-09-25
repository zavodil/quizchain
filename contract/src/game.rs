use crate::*;
use near_sdk::json_types::ValidAccountId;

#[near_bindgen]
impl QuizChain {
    pub fn start_game(&mut self, quiz_id: QuizId) {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            assert_eq!(quiz.status, Status::InProgress, "Quiz is not active");

            let game_id = QuizChain::get_quiz_by_user(quiz_id, env::predecessor_account_id());
            assert!(self.games.get(&game_id).is_none(), "Game already in progress");

            self.games.insert(&game_id,
                              &Game {
                                  answers_quantity: 0,
                                  current_hash: QuizChain::get_hash(quiz.secret.unwrap()),
                              });
        }
    }

    pub fn get_game(&self, quiz_id: QuizId, account_id: ValidAccountId) -> Option<Game> {
        self.games.get(&QuizChain::get_quiz_by_user(quiz_id, account_id.into()))
    }
}