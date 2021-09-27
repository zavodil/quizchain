use crate::*;
use near_sdk::json_types::ValidAccountId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AnswerOutput {
    id: AnswerId,
    selected_option_ids: Option<Vec<QuestionOptionId>>,
    selected_text: Option<String>,
    timestamp: Timestamp,
    is_correct: Option<bool>
}

#[near_bindgen]
impl QuizChain {
    pub fn start_game(&mut self, quiz_id: QuizId) {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            assert_eq!(quiz.status, QuizStatus::InProgress, "Quiz is not active");

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

    pub fn get_answer(&self, quiz_id: QuizId, question_id: QuestionId, account_id: ValidAccountId) -> Option<Answer> {
        self.answers.get(&QuizChain::get_answer_by_quiz_by_question(quiz_id, question_id, account_id.into()))
    }

    /*
    pub fn get_revealed_answer(&self, quiz_id: QuizId, question_id: QuestionId) -> Option<RevealedAnswer> {
        if let Some(quiz) = self.quizzes.get(&quiz_id){
            if let Some(revealed_answers) = quiz.revealed_answers{
                Some(revealed_answers[question_id as usize])
            }
            else {
                None
            }
        }
        else {
            None
        }
    }
    */

    pub fn get_answers(&self, quiz_id: QuizId, account_id: ValidAccountId) -> Vec<AnswerOutput> {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            let revealed_answers = if let Some(revealed_answers_unwrapped) = quiz.revealed_answers {
                revealed_answers_unwrapped
            }
            else{
                Vec::new()
            };
            let revealed_answers_found = revealed_answers.len() > 0;

            let mut answers: Vec<AnswerOutput> = Vec::new();
            let questions = self.get_questions_by_quiz(quiz_id);
            for question in &questions {
                let answer = self.answers.get(
                    &QuizChain::get_answer_by_quiz_by_question(quiz_id, question.id, account_id.clone().into()));
                if let Some(answer_unwrapped) = answer {

                    let is_correct = if revealed_answers_found {
                        let revealed_answer = &revealed_answers[(question.id as usize)];
                        Some(
                            answer_unwrapped.selected_option_ids == revealed_answer.selected_option_ids &&
                            answer_unwrapped.selected_text == revealed_answer.selected_text)
                    }
                    else{
                        None
                    };

                    answers.push(AnswerOutput {
                        id: question.id,
                        selected_option_ids: answer_unwrapped.selected_option_ids,
                        selected_text: answer_unwrapped.selected_text,
                        timestamp: answer_unwrapped.timestamp,
                        is_correct
                    })
                }
            }

            answers
        }
        else{
            panic!("Wrong quiz")
        }
    }

}