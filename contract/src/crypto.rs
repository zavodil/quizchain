use crate::*;

#[near_bindgen]
impl QuizChain {
    pub fn send_answer(&mut self, quiz_id: QuizId, question_id: QuestionId, question_option_id: QuestionOptionId){
        let game_id = QuizChain::get_quiz_by_user(quiz_id, env::predecessor_account_id());
        if let Some(mut game) = self.games.get(&game_id) {
            assert_eq!(question_id, game.answers_quantity, "Wrong index of the answer");

            if let Some(quiz) = self.quizzes.get(&quiz_id) {
                assert_eq!(quiz.status, Status::InProgress, "Quiz is not active");

                if let Some(question_option) = self.question_options.get(&QuizChain::get_question_option_by_quiz(quiz_id, question_id, question_option_id)) {
                    game.answers_quantity += 1;
                    let concat_hash = format!("{}{}{}", game.current_hash, question_option.content, quiz.secret.unwrap());
                    let new_hash = QuizChain::get_hash(concat_hash);
                    game.current_hash = new_hash.clone();
                    self.games.insert(&game_id, &game);
                    log!("Answer added. New game hash: {}", new_hash);
                }
                else{
                    panic!("Question option not found");
                }
            }
            else{
                panic!("Quiz not found");
            }
        }
        else{
            panic!("Game wasn't started");
        }
    }

    pub(crate) fn get_hash(text: String) -> String {
        format!("{:x}", Sha256::digest(text.as_bytes()))
    }

    /*
    pub fn pub_get_hash(&self, text: String) -> String { // TODO remove
        //env::sha256(text.as_bytes())
        //Sha256::digest(text.as_bytes())
        format!("{:x}", Sha256::digest(text.as_bytes()))
    }

    pub fn pub_add_hash(&self, hash: String, text: String) -> String { // TODO remove
        //env::sha256(text.as_bytes())
        //Sha256::digest(text.as_bytes())
        let query = format!("{}{}", hash, text);
        format!("{:x}", Sha256::digest(query.as_bytes()))
    }
     */

    /*pub fn pub_add_hash(&self, hash: Vec<u8>, text: String) -> Vec<u8> { // TODO remove
        env::sha256(hash + text.as_bytes())
    }*/
}