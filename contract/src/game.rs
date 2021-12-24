use std::cmp::min;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AnswerOutput {
    id: AnswerId,
    selected_option_ids: Option<Vec<QuestionOptionId>>,
    selected_text: Option<String>,
    timestamp: Timestamp,
    is_correct: Option<bool>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StatsOutput {
    player_id: AccountId,
    answers_quantity: u16,
    last_answer_timestamp: Option<Timestamp>
}

#[near_bindgen]
impl QuizChain {
    pub fn send_answer(&mut self, quiz_id: QuizId, question_id: QuestionId,
                       question_option_ids: Option<Vec<QuestionOptionId>>,
                       question_option_text: Option<String>) {
        let game_id = QuizChain::get_quiz_by_user(quiz_id, env::predecessor_account_id());
        if let Some(mut game) = self.games.get(&game_id) {
            assert_eq!(question_id, game.answers_quantity, "Wrong index of the answer");

            if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
                QuizChain::assert_game_available_to_play(&quiz.status);

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
                    //log!("Answer '{}' added. New game hash: {}", answer_to_hash, new_hash);

                    if quiz.status == QuizStatus::InProgress && game.answers_quantity == quiz.total_questions {
                        match quiz.finality_type {
                            QuizFinalityType::Direct => self.finalize_game(&game, &quiz_id, &mut quiz),
                            QuizFinalityType::DelayedReveal => self.stop_game(game.current_hash, &quiz_id),
                        };
                    }
                } else {
                    panic!("Question not found");
                }
            } else {
                panic!("Quiz not found");
            }
        } else {
            panic!("Game wasn't started");
        }
    }

    pub(crate) fn stop_game(&mut self, hash: Hash, quiz_id: &QuizId) {
        let index = &QuizResultByQuiz {
            quiz_id: *quiz_id,
            hash,
        };
        let mut accounts_with_same_result = self.quiz_results.get(index).unwrap_or([].to_vec());

        accounts_with_same_result.push(env::predecessor_account_id());

        self.quiz_results.insert(index, &accounts_with_same_result);
    }

    pub(crate) fn finalize_game(&mut self, game: &Game, quiz_id: &QuizId, quiz: &mut Quiz) {
        if game.current_hash == quiz.success_hash.clone().unwrap() {
            //log!("All your answers are valid!");
            if quiz.available_rewards_ids.len() > 0 {
                if let Some((reward_id, other_reward_ids)) = quiz.available_rewards_ids.clone().split_first() {
                    let reward_index = QuizChain::get_reward_by_quiz(*quiz_id, *reward_id);
                    if let Some(mut reward) = self.rewards.get(&reward_index) {
                        assert!(reward.winner_account_id.is_none(), "Reward already distributed");
                        reward.winner_account_id = Some(env::predecessor_account_id());
                        self.rewards.insert(&reward_index, &reward);
                        //log!("Congratulations! You allowed to claim reward of {} yNEAR", reward.amount);

                        quiz.available_rewards_ids = other_reward_ids.to_vec();
                        if other_reward_ids.len() == 0 {
                            quiz.status = QuizStatus::Finished;
                            self.active_quizzes.remove(quiz_id);
                        }
                        quiz.distributed_rewards_ids.push(*reward_id);
                        self.quizzes.insert(quiz_id, &quiz);
                    }
                }

            }
        }
        else {
            //log!("Something was wrong...");
        }
    }

    pub(crate) fn get_hash(text: String) -> String {
        format!("{:x}", Sha256::digest(text.as_bytes()))
    }

    pub (crate) fn assert_game_available_to_play(status: &QuizStatus){
        assert!([QuizStatus::InProgress, QuizStatus::Finished].contains(&status), "Quiz is not active");
    }

    pub fn start_game(&mut self, quiz_id: QuizId, referrer_id: Option<ValidAccountId>) {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            QuizChain::assert_game_available_to_play(&quiz.status);
            let account_id = env::predecessor_account_id();

            let game_id = QuizChain::get_quiz_by_user(quiz_id, account_id.clone());
            assert!(self.games.get(&game_id).is_none(), "Game already in progress");

            let mut players: UnorderedSet<AccountId> = self.players.get(&quiz_id).unwrap_or(UnorderedSet::new(quiz_id.to_string().as_bytes().to_vec()));
            players.insert(&account_id);
            self.players.insert(&quiz_id, &players);
            self.add_quiz_for_player(&quiz_id, account_id.clone());

            if let Some(valid_referrer_account_id) = referrer_id {
                let referrer_id_value: AccountId = valid_referrer_account_id.into();
                if referrer_id_value != account_id && env::is_valid_account_id(referrer_id_value.as_bytes()) {
                    self.internal_increase_referrer_stats(quiz_id, referrer_id_value);
                }
            }

            self.games.insert(&game_id,
                              &Game {
                                  answers_quantity: 0,
                                  current_hash: QuizChain::get_hash(quiz.secret.unwrap()),
                              });
        }
    }

    fn internal_increase_referrer_stats(&mut self, quiz_id: QuizId, referrer_id: AccountId) {
        let mut already_invited_to_this_quiz = if let Some(already_invited) = self.affiliates.get(&quiz_id) {
            already_invited
        } else {
            UnorderedMap::new(StorageKey::AffiliatesByQuiz {
                quiz_id,
            })
        };

        let already_invited_by_referrer = already_invited_to_this_quiz.get(&referrer_id).unwrap_or(0);
        already_invited_to_this_quiz.insert(&referrer_id, &(already_invited_by_referrer + 1));
        self.affiliates.insert(&quiz_id, &already_invited_to_this_quiz);

        let already_invited_total = self.total_affiliates.get(&referrer_id).unwrap_or(0);
        self.total_affiliates.insert(&referrer_id, &(already_invited_total + 1));
    }

    pub fn restart_game(&mut self, quiz_id: QuizId) {
        let account_id = env::predecessor_account_id();
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            assert!(quiz.restart_allowed, "Restarts are now allowed for this quiz");
            assert_eq!(quiz.status, QuizStatus::InProgress, "Quiz is not active");

            let game_id = QuizChain::get_quiz_by_user(quiz_id, account_id.clone());

            if let Some(game) = self.games.get(&game_id) {
                assert_eq!(game.answers_quantity, quiz.total_questions, "Current game is not finished");

                for reward_id in &quiz.distributed_rewards_ids {
                    if let Some(reward) = self.rewards.get(&QuizChain::get_reward_by_quiz(quiz_id, *reward_id)) {
                        if let Some(winner_account_id) = reward.winner_account_id {
                            assert_eq!(winner_account_id, account_id, "Winner is now allowed to restart the game");
                        }
                    }
                }

                self.games.insert(&game_id,
                                  &Game {
                                      answers_quantity: 0,
                                      current_hash: QuizChain::get_hash(quiz.secret.unwrap()),
                                  });

                log!("Game restarted");
            }
            else {
                panic!("Game not found");
            }
        }
    }

    // Test reasons only TODO remove
    #[private]
    pub fn start_game_for_account_id(&mut self, quiz_id: QuizId, account_id: AccountId) {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            let game_id = QuizChain::get_quiz_by_user(quiz_id, account_id.clone());

            let mut players: UnorderedSet<AccountId> = self.players.get(&quiz_id).unwrap_or(UnorderedSet::new(quiz_id.to_string().as_bytes().to_vec()));

            players.insert(&account_id);
            self.players.insert(&quiz_id, &players);

            self.games.insert(&game_id,
                              &Game {
                                  answers_quantity: 0,
                                  current_hash: QuizChain::get_hash(quiz.secret.unwrap()),
                              });
        }
    }

    pub fn get_quiz_stats(&self, quiz_id: QuizId, from_index: usize, limit: usize) -> Option<Vec<StatsOutput>> {
        if let Some(player_ids) = self.players.get(&quiz_id) {
            let player_ids_qty = player_ids.len() as usize;
            let mut stats: Vec<StatsOutput> = Vec::new();
            assert!(from_index <= player_ids_qty, "Illegal from_index");
            let limit_id = min(from_index + limit, player_ids_qty);
            let player_account_ids = player_ids.as_vector();
            for player_index in from_index..limit_id {
                if let Some(player_id) = player_account_ids.get(player_index as u64) {
                    if let Some(game) = self.games.get(&QuizChain::get_quiz_by_user(quiz_id, player_id.clone())) {
                        let last_answer_timestamp = if game.answers_quantity > 0 {
                            let last_answer = self.answers.get(&QuizChain::get_answer_by_quiz_by_question(quiz_id, game.answers_quantity - 1, player_id.clone()));
                            if let Some(last_answer_value) = last_answer {
                                Some(last_answer_value.timestamp)
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        stats.push(StatsOutput {
                            player_id,
                            answers_quantity: game.answers_quantity,
                            last_answer_timestamp,
                        });
                    }
                }
            }
            Some(stats)
        } else {
            None
        }
    }

    pub fn get_game(&self, quiz_id: QuizId, account_id: ValidAccountId) -> Option<Game> {
        self.games.get(&QuizChain::get_quiz_by_user(quiz_id, account_id.into()))
    }

    pub fn get_answer(&self, quiz_id: QuizId, question_id: QuestionId, account_id: ValidAccountId) -> Option<Answer> {
        self.answers.get(&QuizChain::get_answer_by_quiz_by_question(quiz_id, question_id, account_id.into()))
    }

    pub fn get_revealed_answer(&self, quiz_id: QuizId, question_id: QuestionId) -> Option<RevealedAnswer> {
        if let Some(quiz) = self.quizzes.get(&quiz_id){
            if let Some(revealed_answers) = quiz.revealed_answers{
                Some(revealed_answers[question_id as usize].clone())
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

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
