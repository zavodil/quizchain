use near_sdk::json_types::ValidAccountId;

use crate::*;

const DAY_IN_NANOSECONDS: Timestamp = 86400000000000;
const DAYS_BEFORE_CANCEL: u64 = 5;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuizOutput {
    id: QuizId,
    title: Option<String>,
    description: Option<String>,
    language: Option<String>,
    finality_type: QuizFinalityType,
    owner_id: AccountId,
    status: QuizStatus,
    total_questions: u16,
    unclaimed_rewards_ids: Vec<RewardId>,
    secret: Option<String>,
    success_hash: Option<String>,
    questions: Vec<QuestionOutput>,
    available_rewards: Vec<RewardOutput>,
    distributed_rewards: Vec<RewardOutput>,
    revealed_answers: Option<Vec<RevealedAnswer>>,
    timestamp: Option<Timestamp>,
    restart_allowed: bool,
    token_account_id: Option<TokenAccountId>,
    funded_amount: Option<Balance>
}

// 10 NEAR
const MAX_SERVICE_FEE: Balance = 10_000_000_000_000_000_000_000_000;
const SERVICE_RATE_NUMERATOR: Balance = 1;
const SERVICE_RATE_DENOMINATOR: Balance = 100;

#[near_bindgen]
impl QuizChain {
    #[payable]
    pub fn create_quiz_for_account(&mut self, quiz_owner_id: ValidAccountId, token_account_id: Option<TokenAccountId>) -> QuizId {
        let deposit = env::attached_deposit();

        self.create_quiz_for_account_internal(env::predecessor_account_id(), quiz_owner_id.into(), deposit, token_account_id)
    }

    pub(crate) fn create_quiz_for_account_internal(
        &mut self, sender_id: AccountId,
        quiz_owner_id: AccountId,
        deposit: Balance,
        token_account_id: Option<TokenAccountId>)
        -> QuizId {

        let service_fee: Balance =
                deposit - deposit * SERVICE_RATE_DENOMINATOR / (SERVICE_RATE_DENOMINATOR + SERVICE_RATE_NUMERATOR) ;
        let funded_amount: Balance = deposit - service_fee;

        self.add_service_fees_total(service_fee, &token_account_id);

        let quiz_id = self.next_quiz_id;
        self.quizzes.insert(&quiz_id,
                            &Quiz {
                                title: None,
                                description: None,
                                language: None,
                                finality_type: QuizFinalityType::Direct,
                                owner_id: quiz_owner_id.clone().into(),
                                status: QuizStatus::Funded,
                                total_questions: 0,
                                available_rewards_ids: Vec::new(),
                                distributed_rewards_ids: Vec::new(),
                                secret: None,
                                success_hash: None,
                                revealed_answers: None,
                                sponsor_account_id: Some(sender_id.clone()),
                                funded_amount: Some(funded_amount),
                                restart_allowed: false,
                                timestamp: Some(env::block_timestamp()),
                                token_account_id,
                            });

        self.add_quiz_for_owner(&quiz_id, quiz_owner_id.into());
        self.add_quiz_for_sponsor(&quiz_id, sender_id);

        self.next_quiz_id += 1;
        quiz_id
    }

    pub fn update_funded_quiz(&mut self,
                              quiz_id: QuizId,
                              title: String,
                              description: Option<String>,
                              language: Option<String>,
                              finality_type: QuizFinalityType,
                              questions: Vec<QuestionInput>,
                              all_question_options: Vec<Vec<QuestionOption>>,
                              rewards: Vec<RewardInput>,
                              secret: Option<String>,
                              success_hash: Option<String>,
                              restart_allowed: bool) -> QuizId {
        assert_eq!(questions.len(), all_question_options.len(), "Questions and question options not matched");
        assert!(!questions.is_empty(), "Data not found");
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            assert_eq!(quiz.status, QuizStatus::Funded);
            let mut unclaimed_rewards_ids = Vec::new();
            let mut rewards_total: Balance = 0;

            for (index, reward) in rewards.iter().enumerate() {
                rewards_total += reward.amount.0;
                let reward_id: RewardId = index as RewardId;
                self.rewards.insert(
                    &QuizChain::get_reward_by_quiz(quiz_id, reward_id),
                    &Reward {
                        amount: reward.amount.0,
                        winner_account_id: None,
                        claimed: false,
                    });
                unclaimed_rewards_ids.push(reward_id);
            }

            let funded_amount = quiz.funded_amount.unwrap_or(0);
            assert_eq!(funded_amount, rewards_total,
                       "Illegal rewards. Total available rewards: {} yNEAR", funded_amount);


            let total_questions = questions.len() as u16;

            let mut options_quantity = 0;

            let mut question_id: QuestionId = 0;
            for question in &questions {
                if let Some(question_options) = all_question_options.get(question_id as usize) {
                    let mut question_option_id: QuestionOptionId = 0;
                    for question_option in question_options {
                        self.question_options.insert(
                            &QuizChain::get_question_option_by_quiz(quiz_id, question_id, question_option_id),
                            &question_option);
                        question_option_id += 1;
                    }

                    options_quantity = question_options.len() as u16;
                }

                self.questions.insert(
                    &QuizChain::get_question_by_quiz(quiz_id, question_id),
                    &Question {
                        content: question.content.clone(),
                        hint: question.hint.clone(),
                        options_quantity,
                        kind: question.kind,
                    });

                question_id += 1;
            }

            let quiz = Quiz {
                title: Some(title),
                description,
                language,
                finality_type,
                owner_id: env::predecessor_account_id(),
                status: QuizStatus::Locked,
                total_questions,
                available_rewards_ids: unclaimed_rewards_ids,
                distributed_rewards_ids: Vec::new(),
                secret: secret.clone(),
                success_hash: success_hash.clone(),
                revealed_answers: None,
                sponsor_account_id: None,
                funded_amount: None,
                restart_allowed,
                timestamp: Some(env::block_timestamp()),
                token_account_id: quiz.token_account_id,
            };
            self.quizzes.insert(&quiz_id, &quiz);
            self.add_quiz_for_owner(&quiz_id, env::predecessor_account_id());

            if let Some(secret_unwrapped) = secret {
                self.activate_quiz(quiz_id, secret_unwrapped, success_hash)
            }

            quiz_id
        } else {
            panic!("Quiz not found");
        }
    }
    pub(crate) fn add_quiz_for_player(&mut self, quiz_id: &QuizId, account_id: AccountId) {
        let mut quizzes_for_player: Vec<QuizId> = self.quizzes_by_player_id.get(&account_id).unwrap_or([].to_vec());
        if !quizzes_for_player.contains(quiz_id) {
            quizzes_for_player.push(*quiz_id);
            self.quizzes_by_player_id.insert(&account_id, &quizzes_for_player);
        }
    }

    pub(crate) fn add_quiz_for_owner(&mut self, quiz_id: &QuizId, account_id: AccountId) {
        let mut quizzes_for_owner: Vec<QuizId> = self.quizzes_by_owner_id.get(&account_id).unwrap_or([].to_vec());
        if !quizzes_for_owner.contains(quiz_id) {
            quizzes_for_owner.push(*quiz_id);
            self.quizzes_by_owner_id.insert(&account_id, &quizzes_for_owner);
        }
    }

    pub(crate) fn add_quiz_for_sponsor(&mut self, quiz_id: &QuizId, account_id: AccountId) {
        let mut quizzes_for_sponsor: Vec<QuizId> = self.quizzes_by_sponsor_id.get(&account_id).unwrap_or([].to_vec());
        if !quizzes_for_sponsor.contains(quiz_id) {
            quizzes_for_sponsor.push(*quiz_id);
            self.quizzes_by_sponsor_id.insert(&account_id, &quizzes_for_sponsor);
        }
    }

    pub (crate) fn get_quizzes_by_ids(&self, quizzes_ids: Vec<QuizId>, from_index: usize, limit: usize) -> Vec<QuizOutput>{
        let quizzes_ids_qty = quizzes_ids.len() as usize;
        let mut quizzes: Vec<QuizOutput> = Vec::new();
        assert!(from_index <= quizzes_ids_qty, "Illegal from_index");
        let limit_id = std::cmp::min(from_index + limit, quizzes_ids_qty);
        for quiz_index in from_index..limit_id {
            if let Some(quiz) = self.get_quiz(quizzes_ids[quiz_index]) {
                quizzes.push(quiz);
            }
        }
        quizzes
    }

    pub fn get_quizzes_by_player(&self, account_id: ValidAccountId, from_index: usize, limit: usize) -> Option<Vec<QuizOutput>> {
        if let Some(quizzes_ids) = self.quizzes_by_player_id.get(&account_id.into()) {
            Some(self.get_quizzes_by_ids(quizzes_ids, from_index, limit))
        } else {
            None
        }
    }

    pub fn get_quizzes_by_owner(&self, account_id: ValidAccountId, from_index: usize, limit: usize) -> Option<Vec<QuizOutput>> {
        if let Some(quizzes_ids) = self.quizzes_by_owner_id.get(&account_id.into()) {
            Some(self.get_quizzes_by_ids(quizzes_ids, from_index, limit))
        } else {
            None
        }
    }

    pub fn get_quizzes_by_sponsor(&self, account_id: ValidAccountId, from_index: usize, limit: usize) -> Option<Vec<QuizOutput>> {
        if let Some(quizzes_ids) = self.quizzes_by_sponsor_id.get(&account_id.into()) {
            Some(self.get_quizzes_by_ids(quizzes_ids, from_index, limit))
        } else {
            None
        }
    }

    pub fn get_total_affiliates_for_account(&self, account_id: AccountId) -> u64 {
        self.total_affiliates.get(&account_id).unwrap_or(0)
    }

    pub fn get_total_affiliates(&self, from_index: u64, limit: u64) -> Vec<AffiliatesOutput> {
        let keys = self.total_affiliates.keys_as_vector();
        let values = self.total_affiliates.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, keys.len())).map(|index| {
            AffiliatesOutput {
                account_id: keys.get(index).unwrap(),
                affiliates: values.get(index).unwrap(),
            }
        }).collect()
    }

    pub fn get_affiliates_for_account(&self, account_id: AccountId, quiz_id: QuizId) -> u64 {
        if let Some(get_affiliates_for_account_value) = self.internal_get_affiliates_by_quiz(&quiz_id){
            get_affiliates_for_account_value.get(&account_id).unwrap_or(0)
        }
        else{
            0
        }
    }

    pub fn get_affiliates(&self, quiz_id: QuizId, from_index: u64, limit: u64) -> Vec<AffiliatesOutput> {
        if let Some(affiliates_by_quiz) = self.internal_get_affiliates_by_quiz(&quiz_id) {
            let keys = affiliates_by_quiz.keys_as_vector();
            let values = affiliates_by_quiz.values_as_vector();
            (from_index..std::cmp::min(from_index + limit, keys.len())).map(|index| {
                AffiliatesOutput {
                    account_id: keys.get(index).unwrap(),
                    affiliates: values.get(index).unwrap(),
                }
            }).collect()
        }
        else {
            [].to_vec()
        }
    }

    fn internal_get_affiliates_by_quiz(&self, quiz_id: &QuizId) -> Option<UnorderedMap<AccountId, u64>> {
        self.affiliates.get(quiz_id)
    }

    #[payable]
    pub fn create_quiz(&mut self,
                       title: String,
                       description: Option<String>,
                       language: Option<String>,
                       finality_type: QuizFinalityType,
                       questions: Vec<QuestionInput>,
                       all_question_options: Vec<Vec<QuestionOption>>,
                       rewards: Vec<RewardInput>,
                       secret: Option<String>,
                       success_hash: Option<String>,
                       restart_allowed: bool,
                       token_account_id: Option<TokenAccountId>) -> QuizId {
        let deposit = env::attached_deposit();
        let owner_id = env::predecessor_account_id();

        let quiz_id = self.create_quiz_internal(owner_id,
                                                title,
                                                description,
                                                language,
                                                finality_type,
                                                questions,
                                                all_question_options,
                                                rewards,
                                                secret.clone(),
                                                success_hash.clone(),
                                                restart_allowed,
                                                deposit,
                                                token_account_id);

        if let Some(secret_unwrapped) = secret {
            self.activate_quiz(quiz_id, secret_unwrapped, success_hash);
        }

        quiz_id
    }

    pub fn create_quiz_internal(&mut self,
                                owner_id: AccountId,
                                title: String,
                                description: Option<String>,
                                language: Option<String>,
                                finality_type: QuizFinalityType,
                                questions: Vec<QuestionInput>,
                                all_question_options: Vec<Vec<QuestionOption>>,
                                rewards: Vec<RewardInput>,
                                secret: Option<String>,
                                success_hash: Option<String>,
                                restart_allowed: bool,
                                deposit: Balance,
                                token_account_id: Option<TokenAccountId>) -> QuizId {
        assert_eq!(questions.len(), all_question_options.len(), "Questions and question options not matched");
        assert!(!questions.is_empty(), "Data not found");

        let quiz_id = self.next_quiz_id;

        let mut reward_id: RewardId = 0;
        let mut unclaimed_rewards_ids = Vec::new();
        let mut rewards_total: Balance = 0;

        for reward in &rewards {
            rewards_total += reward.amount.0;
            self.rewards.insert(
                &QuizChain::get_reward_by_quiz(quiz_id, reward_id),
                &Reward {
                    amount: reward.amount.0,
                    winner_account_id: None,
                    claimed: false,
                });
            unclaimed_rewards_ids.push(reward_id);

            reward_id += 1;
        }

        let service_fee: Balance = if QuizChain::unwrap_token_id(&token_account_id) == NEAR.to_string() {
            std::cmp::min(rewards_total * SERVICE_RATE_NUMERATOR / SERVICE_RATE_DENOMINATOR, MAX_SERVICE_FEE)
        } else {
            rewards_total * SERVICE_RATE_NUMERATOR / SERVICE_RATE_DENOMINATOR
        };
        assert_eq!(deposit, rewards_total + service_fee,
                   "Illegal deposit, please deposit {} yNEAR for rewards and {} yNEAR for the service fee", rewards_total, service_fee);

        self.add_service_fees_total(service_fee, &token_account_id);

        self.next_quiz_id += 1;
        let total_questions = questions.len() as u16;

        let mut options_quantity = 0;

        let mut question_id: QuestionId = 0;
        for question in &questions {
            if let Some(question_options) = all_question_options.get(question_id as usize) {
                let mut question_option_id: QuestionOptionId = 0;
                for question_option in question_options {
                    self.question_options.insert(
                        &QuizChain::get_question_option_by_quiz(quiz_id, question_id, question_option_id),
                        question_option);
                    question_option_id += 1;
                }

                options_quantity = question_options.len() as u16;
            }

            self.questions.insert(
                &QuizChain::get_question_by_quiz(quiz_id, question_id),
                &Question{
                    content: question.content.clone(),
                    hint: question.hint.clone(),
                    options_quantity,
                    kind: question.kind
                });

            question_id += 1;
        }

        let quiz = Quiz {
            title: Some(title),
            description,
            language,
            finality_type,
            owner_id: owner_id.clone(),
            status: QuizStatus::Locked,
            total_questions,
            available_rewards_ids: unclaimed_rewards_ids,
            distributed_rewards_ids: Vec::new(),
            secret,
            success_hash,
            revealed_answers: None,
            sponsor_account_id: None,
            funded_amount: None,
            restart_allowed,
            timestamp: Some(env::block_timestamp()),
            token_account_id,
        };
        self.quizzes.insert(&quiz_id, &quiz);

        self.add_quiz_for_owner(&quiz_id, owner_id);

        quiz_id
    }

    pub fn create_quiz_and_activate_internal(&mut self,
                                owner_id: AccountId,
                                title: String,
                                description: Option<String>,
                                language: Option<String>,
                                finality_type: QuizFinalityType,
                                questions: Vec<QuestionInput>,
                                all_question_options: Vec<Vec<QuestionOption>>,
                                rewards: Vec<RewardInput>,
                                secret: Secret,
                                success_hash: Option<Hash>,
                                restart_allowed: bool,
                                deposit: Balance,
                                token_account_id: Option<TokenAccountId>) -> QuizId {
        assert_eq!(questions.len(), all_question_options.len(), "Questions and question options not matched");
        assert!(!questions.is_empty(), "Data not found");

        let quiz_id = self.next_quiz_id;

        let mut reward_id: RewardId = 0;
        let mut unclaimed_rewards_ids = Vec::new();
        let mut rewards_total: Balance = 0;

        for reward in &rewards {
            rewards_total += reward.amount.0;
            self.rewards.insert(
                &QuizChain::get_reward_by_quiz(quiz_id, reward_id),
                &Reward {
                    amount: reward.amount.0,
                    winner_account_id: None,
                    claimed: false,
                });
            unclaimed_rewards_ids.push(reward_id);

            reward_id += 1;
        }

        let service_fee: Balance = if QuizChain::unwrap_token_id(&token_account_id) == NEAR.to_string() {
            std::cmp::min(rewards_total * SERVICE_RATE_NUMERATOR / SERVICE_RATE_DENOMINATOR, MAX_SERVICE_FEE)
        } else {
            rewards_total * SERVICE_RATE_NUMERATOR / SERVICE_RATE_DENOMINATOR
        };
        assert_eq!(deposit, rewards_total + service_fee,
                   "Illegal deposit, please deposit {} yNEAR for rewards and {} yNEAR for the service fee", rewards_total, service_fee);

        self.add_service_fees_total(service_fee, &token_account_id);

        self.next_quiz_id += 1;
        let total_questions = questions.len() as u16;

        let mut options_quantity = 0;

        let mut question_id: QuestionId = 0;
        for question in &questions {
            if let Some(question_options) = all_question_options.get(question_id as usize) {
                let mut question_option_id: QuestionOptionId = 0;
                for question_option in question_options {
                    self.question_options.insert(
                        &QuizChain::get_question_option_by_quiz(quiz_id, question_id, question_option_id),
                        question_option);
                    question_option_id += 1;
                }

                options_quantity = question_options.len() as u16;
            }

            self.questions.insert(
                &QuizChain::get_question_by_quiz(quiz_id, question_id),
                &Question{
                    content: question.content.clone(),
                    hint: question.hint.clone(),
                    options_quantity,
                    kind: question.kind
                });

            question_id += 1;
        }

        let quiz = Quiz {
            title: Some(title),
            description,
            language,
            finality_type,
            owner_id: owner_id.clone(),
            status: QuizStatus::InProgress,
            total_questions,
            available_rewards_ids: unclaimed_rewards_ids,
            distributed_rewards_ids: Vec::new(),
            secret: Some(secret),
            success_hash,
            revealed_answers: None,
            sponsor_account_id: None,
            funded_amount: None,
            restart_allowed,
            timestamp: Some(env::block_timestamp()),
            token_account_id,
        };
        self.quizzes.insert(&quiz_id, &quiz);

        self.active_quizzes.insert(&quiz_id);

        self.add_quiz_for_owner(&quiz_id, owner_id);

        quiz_id
    }

    pub fn activate_quiz(&mut self, quiz_id: QuizId, secret: Secret, success_hash: Option<Hash>) {
        self.activate_quiz_internal(env::predecessor_account_id(), quiz_id, secret, success_hash);
    }

    pub(crate) fn activate_quiz_internal(&mut self, quiz_owner_id: AccountId, quiz_id: QuizId, secret: Secret, success_hash: Option<Hash>) {
        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            assert_eq!(quiz.owner_id, quiz_owner_id, "Not a quiz owner");
            assert_eq!(quiz.status, QuizStatus::Locked, "Quiz was already unlocked");

            quiz.secret = Some(secret);
            quiz.status = QuizStatus::InProgress;
            quiz.success_hash = success_hash;
            self.quizzes.insert(&quiz_id, &quiz);
            self.active_quizzes.insert(&quiz_id);
        }
    }

    #[private]
    pub fn cancel_quiz(&mut self, quiz_id: QuizId) -> PromiseOrValue<bool> {
        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            assert!([QuizStatus::InProgress, QuizStatus::Locked].contains(&quiz.status), "Quiz is not available to cancel");
            if let Some(timestamp) = quiz.timestamp {
                assert!(env::block_timestamp() - timestamp > DAY_IN_NANOSECONDS * DAYS_BEFORE_CANCEL, "To early to cancel");

                quiz.status = QuizStatus::Finished;
                self.active_quizzes.remove(&quiz_id);

                let available_rewards = self.get_available_rewards(quiz_id);
                return PromiseOrValue::Promise(self.withdraw_available_rewards(available_rewards.0, quiz.owner_id, quiz.token_account_id));
            }
        }
        PromiseOrValue::Value(false)
    }

    pub fn cancel_funded_quiz(&mut self, quiz_id: QuizId) -> PromiseOrValue<bool> {
        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            if let Some(sponsor_account_id) = quiz.sponsor_account_id {
                assert_eq!(sponsor_account_id, env::predecessor_account_id(), "No access. Only sponsors may cancel inactive quizzes");
                assert_eq!(quiz.status, QuizStatus::Funded, "Quiz was updated");
                if let Some(timestamp) = quiz.timestamp {
                    assert!(env::block_timestamp() - timestamp > DAY_IN_NANOSECONDS * DAYS_BEFORE_CANCEL, "To early to cancel");

                    quiz.status = QuizStatus::Finished;
                    self.active_quizzes.remove(&quiz_id);

                    let available_rewards = self.get_available_rewards(quiz_id);

                    return PromiseOrValue::Promise(self.withdraw_available_rewards(available_rewards.0, sponsor_account_id, quiz.token_account_id))
                }
            }
        }
        PromiseOrValue::Value(false)
    }

    #[payable]
    pub fn reveal_answers(&mut self, quiz_id: QuizId, revealed_answers: Vec<RevealedAnswer>) {
        assert_one_yocto();

        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            QuizChain::assert_current_user(&quiz.owner_id);
            assert!(quiz.revealed_answers.is_none(), "Answers were already revealed");
            assert_eq!(quiz.status, QuizStatus::Finished, "Quiz is not finished");
            assert_eq!(quiz.total_questions, revealed_answers.len() as u16, "Illegal answers quantity");

            let mut hash = QuizChain::get_hash(quiz.secret.clone().unwrap());
            let questions = self.get_questions_by_quiz(quiz_id);

            let mut question_id: QuestionId = 0;
            for answer in &revealed_answers {
                let answer_value = if answer.selected_option_ids.is_some(){
                    let question: &QuestionOutput = questions.get(question_id as usize).unwrap();
                    let options_quantity = question.question_options.len() as QuestionId;
                    let options = self.get_question_options_by_question_id(quiz_id, question_id, options_quantity);

                    let mut answer_string: String = "".to_string();
                    let mut option_ids: Vec<QuestionOptionId> = revealed_answers[question_id as usize].selected_option_ids.clone().unwrap();
                    option_ids.sort_unstable();
                    for option_id in &option_ids {
                        answer_string = format!("{}{}", answer_string, options[*option_id as usize].content.to_lowercase());
                    }
                    answer_string
                }
                else {
                    answer.selected_text.clone().unwrap().to_lowercase()
                };

                hash = QuizChain::get_hash(format!("{}{}", hash, answer_value));

                question_id += 1;
            }

            assert_eq!(hash, quiz.success_hash.clone().unwrap(), "Provided answers are not valid");

            quiz.revealed_answers = Some(revealed_answers);
            self.quizzes.insert(&quiz_id, &quiz);
            log!("Provided answers are valid");
        }
    }

    #[payable]
    pub fn reveal_final_hash(&mut self, quiz_id: QuizId, hash: Hash) -> PromiseOrValue<bool> {
        assert_one_yocto();
        assert_eq!(hash.chars().count(), 64, "Illegal hash length");

        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            QuizChain::assert_current_user(&quiz.owner_id);

            assert_eq!(quiz.finality_type, QuizFinalityType::DelayedReveal, "Hash reveal is not supported");
            assert_eq!(quiz.status, QuizStatus::InProgress, "Quiz is not in Progress");

            let winners: Vec<AccountId> = self.quiz_results.get(&QuizResultByQuiz { quiz_id, hash: hash.clone() }).unwrap_or([].to_vec());
            let winners_qty = winners.len() as u16;
            let total_rewards_qty = quiz.available_rewards_ids.len();

            let mut unspent_rewards: Balance = 0;

            for reward_id in 0..total_rewards_qty as RewardId {
                let reward_index = QuizChain::get_reward_by_quiz(quiz_id, reward_id);
                if let Some(mut reward) = self.rewards.get(&reward_index) {
                    if reward_id < winners_qty {
                        assert!(reward.winner_account_id.is_none(), "Reward already distributed");
                        let winner_account_id = winners[reward_id as usize].clone();
                        reward.winner_account_id = Some(winner_account_id);
                        self.rewards.insert(&reward_index, &reward);
                        quiz.distributed_rewards_ids.push(reward_id);
                    } else {
                        unspent_rewards += reward.amount;
                    }
                }
            }

            quiz.available_rewards_ids = [].to_vec();
            quiz.status = QuizStatus::Finished;
            quiz.success_hash = Some(hash);
            self.active_quizzes.remove(&quiz_id);
            self.quizzes.insert(&quiz_id, &quiz);

            if unspent_rewards > 0 {
                PromiseOrValue::Promise(self.withdraw_available_rewards(unspent_rewards, quiz.owner_id, quiz.token_account_id))
            } else {
                PromiseOrValue::Value(true)
            }
        } else {
            PromiseOrValue::Value(false)
        }
    }

    pub fn get_users_with_final_hash(&self, quiz_id: QuizId, hash: Hash) -> Option<Vec<AccountId>> {
        return self.quiz_results.get(&QuizResultByQuiz { quiz_id, hash })
    }

    pub fn get_available_rewards(&self, quiz_id: QuizId) -> WrappedBalance {
        let mut unspent_rewards: Balance = 0;
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            assert_eq!(quiz.status, QuizStatus::InProgress, "Quiz is not in Progress");

            for reward_id in 0..quiz.available_rewards_ids.len() as RewardId {
                let reward_index = QuizChain::get_reward_by_quiz(quiz_id, reward_id);
                if let Some(reward) = self.rewards.get(&reward_index) {
                    unspent_rewards += reward.amount;
                }
            }
        }
        unspent_rewards.into()
    }

    pub(crate) fn withdraw_available_rewards(&mut self,
                                             available_rewards: Balance,
                                             recipient_account_id: AccountId,
                                             token_account_id: Option<TokenAccountId>) -> Promise {
        let service_fee: Balance = available_rewards / 10;
        self.add_service_fees_total(service_fee, &token_account_id);
        let token_account_id_unwrapped = QuizChain::unwrap_token_id(&token_account_id);
        log!("Unspent rewards: {} of {} found. {} goes to the bank", available_rewards, token_account_id_unwrapped, service_fee);

        self.withdraw(recipient_account_id, available_rewards - service_fee, token_account_id, None, None)
    }

    pub(crate) fn assert_current_user(owner_id: &AccountId) {
        assert_eq!(*owner_id, env::predecessor_account_id(), "No access");
    }

    pub fn get_quiz(&self, quiz_id: QuizId) -> Option<QuizOutput> {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            Some(QuizOutput {
                id: quiz_id,
                title: quiz.title,
                description: quiz.description,
                language: quiz.language,
                finality_type: quiz.finality_type,
                owner_id: quiz.owner_id,
                status: quiz.status,
                total_questions: quiz.total_questions,
                unclaimed_rewards_ids: quiz.available_rewards_ids,
                secret: quiz.secret,
                success_hash: quiz.success_hash,
                questions: self.get_questions_by_quiz(quiz_id),
                available_rewards: self.get_unclaimed_rewards_by_quiz(quiz_id),
                distributed_rewards: self.get_distributed_rewards_by_quiz(quiz_id),
                revealed_answers: quiz.revealed_answers,
                timestamp: quiz.timestamp,
                restart_allowed: quiz.restart_allowed,
                token_account_id: quiz.token_account_id,
                funded_amount: quiz.funded_amount
            })
        }
        else {
            None
        }
    }

    pub fn get_active_quizzes (&self) -> Vec<QuizId> {
        self.active_quizzes.to_vec()
    }

    pub fn get_questions_by_quiz(&self, quiz_id: QuizId) -> Vec<QuestionOutput> {
        let mut questions: Vec<QuestionOutput> = Vec::new();
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            for question_id in 0u16..quiz.total_questions {
                if let Some(question) = self.questions.get(&QuizChain::get_question_by_quiz(quiz_id, question_id)) {
                    questions.push(QuestionOutput {
                        id: question_id,
                        question: question.clone(),
                        question_options: self.get_question_options_by_question_id(quiz_id, question_id, question.options_quantity),
                    });
                }
            }
        }
        questions
    }

    pub fn get_question_options_by_question_id(&self, quiz_id: QuizId, question_id: QuestionId, options_quantity: u16) -> Vec<QuestionOptionOutput> {
        let mut question_options: Vec<QuestionOptionOutput> = Vec::new();

        for question_option_id in 0u16..options_quantity {
            if let Some(question_option) = self.question_options.get(&QuizChain::get_question_option_by_quiz(quiz_id, question_id, question_option_id)) {
                question_options.push(QuestionOptionOutput {
                    id: question_option_id,
                    content: question_option.content,
                    kind: question_option.kind,
                });
            }
        }

        question_options
    }

    #[private]
    #[payable]
    pub fn update_hash_for_finished_quiz_without_answers(&mut self, quiz_id: QuizId, hash: Hash) -> PromiseOrValue<bool>{
        assert_one_yocto();
        assert_eq!(hash.chars().count(), 64, "Illegal hash length");

        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            assert_eq!(quiz.finality_type, QuizFinalityType::DelayedReveal, "Hash reveal is not supported");
            assert!(quiz.revealed_answers.is_none(), "Quiz has answers");
            assert_eq!(quiz.status, QuizStatus::Finished, "Quiz is not Finished");

            quiz.success_hash = Some(hash);
            self.quizzes.insert(&quiz_id, &quiz);
            PromiseOrValue::Value(true)
        } else {
            PromiseOrValue::Value(false)
        }
    }
}
