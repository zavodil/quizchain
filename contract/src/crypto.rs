use crate::*;

#[near_bindgen]
impl QuizChain {
    pub fn send_answer(&mut self, quiz_id: QuizId, question_id: QuestionId,
                       question_option_ids: Option<Vec<QuestionOptionId>>,
                       question_option_text: Option<String>) {
        let game_id = QuizChain::get_quiz_by_user(quiz_id, env::predecessor_account_id());
        if let Some(mut game) = self.games.get(&game_id) {
            assert_eq!(question_id, game.answers_quantity, "Wrong index of the answer");

            if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
                assert_eq!(quiz.status, QuizStatus::InProgress, "Quiz is not active");

                if let Some(question) = self.questions.get(&QuizChain::get_question_by_quiz(quiz_id, question_id)) {
                    let mut answer_to_hash: String = "".to_string();

                    if question.kind == QuestionKind::Text {
                        if let Some(question_option_text_unwrapped) = question_option_text.clone(){
                            answer_to_hash = question_option_text_unwrapped.to_lowercase();
                            self.answers.insert(&QuizChain::get_answer_by_quiz_by_question(quiz_id, question_id, env::predecessor_account_id()),
                                                &Answer {
                                                    selected_option_ids: None,
                                                    selected_text: Some(answer_to_hash.clone()),
                                                    timestamp: env::block_timestamp(),
                                                });
                        }
                        else {
                            panic!("Answer Text is missing");
                        }
                    } else { // OneChoice & MultipleChoice
                        if let Some(mut question_option_ids_unwrapped) = question_option_ids{
                            question_option_ids_unwrapped.sort();
                            for question_option_id in &question_option_ids_unwrapped {
                                if let Some(question_option) = self.question_options.get(
                                    &QuizChain::get_question_option_by_quiz(quiz_id, question_id, *question_option_id)) {
                                    answer_to_hash = format!("{}{}", answer_to_hash, question_option.content).to_lowercase();
                                } else {
                                    panic!("Question option not found");
                                }
                            }
                            self.answers.insert(&QuizChain::get_answer_by_quiz_by_question(quiz_id, question_id, env::predecessor_account_id()),
                                                &Answer {
                                                    selected_option_ids: Some(question_option_ids_unwrapped),
                                                    selected_text: None,
                                                    timestamp: env::block_timestamp(),
                                                });
                        }
                        else{
                            panic!("Answer Options are missing")
                        }
                    }

                    game.answers_quantity += 1;
                    let concat_hash = format!("{}{}", game.current_hash, answer_to_hash.clone());
                    let new_hash = QuizChain::get_hash(concat_hash);
                    game.current_hash = new_hash.clone();
                    self.games.insert(&game_id, &game);
                    log!("Answer '{}' added. New game hash: {}", answer_to_hash, new_hash);

                    if game.answers_quantity == quiz.total_questions {
                        self.finalize_game(&game, &quiz_id, &mut quiz);
                    }
                } else {
                    panic!("Question not found");
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

    pub(crate) fn finalize_game(&mut self, game: &Game, quiz_id: &QuizId, quiz: &mut Quiz){
        if game.current_hash == quiz.success_hash.clone().unwrap() {
            log!("All your answers are valid!");
            if quiz.available_rewards_ids.len() > 0 {
                if let Some((reward_id, other_reward_ids)) = quiz.available_rewards_ids.clone().split_first() {

                    let reward_index = QuizChain::get_reward_by_quiz(*quiz_id, *reward_id);
                    if let Some(mut reward) = self.rewards.get(&reward_index){
                        assert!(reward.winner_account_id.is_none(), "Reward already distributed");
                        reward.winner_account_id = Some(env::predecessor_account_id());
                        self.rewards.insert(&reward_index, &reward);
                        log!("Congratulations! You allowed to claim reward of {} yNEAR", reward.amount);

                        quiz.available_rewards_ids = other_reward_ids.to_vec();
                        if other_reward_ids.len() == 0 {
                            quiz.status = QuizStatus::Finished;
                        }
                        quiz.distributed_rewards_ids.push(*reward_id);
                        self.quizzes.insert(quiz_id, &quiz);
                        self.active_quizzes.remove(quiz_id);
                    }
                }

            }
        }
        else {
            log!("Something was wrong...");
        }
    }



    pub(crate) fn get_hash(text: String) -> String {
        format!("{:x}", Sha256::digest(text.as_bytes()))
    }
}