use crate::*;
use std::cmp::min;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuizOutput {
    owner_id: AccountId,
    status: QuizStatus,
    total_questions: u16,
    unclaimed_rewards_ids: Vec<RewardId>,
    secret: Option<String>,
    success_hash: Option<String>,
    questions: Vec<QuestionOutput>,
    available_rewards: Vec<RewardOutput>,
    distributed_rewards: Vec<RewardOutput>,
    revealed_answers: Option<Vec<RevealedAnswer>>
}

// 10 NEAR
const MAX_SERVICE_FEE: Balance = 10_000_000_000_000_000_000_000_000;

#[near_bindgen]
impl QuizChain {
    #[payable]
    pub fn create_quiz(&mut self,
                       questions: Vec<QuestionInput>,
                       all_question_options: Vec<Vec<QuestionOption>>,
                       rewards: Vec<RewardInput>,
                       secret: Option<String>,
                       success_hash: Option<String>) -> QuizId {
        assert_eq!(questions.len(), all_question_options.len(), "Questions and question options not matched");
        assert!(questions.len() > 0, "Data not found");

        let quiz_id = self.next_quiz_id;

        let mut reward_id: RewardId = 0;
        let mut unclaimed_rewards_ids = Vec::new();
        let mut rewards_total: Balance = 0;

        for reward in &rewards {
            rewards_total += reward.amount.0;
            self.rewards.insert(
                &QuizChain::get_reward_by_quiz(quiz_id, reward_id),
                &Reward{
                    amount: reward.amount.0,
                    winner_account_id: None,
                    claimed: false
                });
            unclaimed_rewards_ids.push(reward_id);

            reward_id += 1;
        }

        let service_fee = min(rewards_total / 100, MAX_SERVICE_FEE);
        assert_eq!(env::attached_deposit(), rewards_total + service_fee,
                   "Illegal deposit, please deposit {} yNEAR for rewards and {} yNEAR for the service fee", rewards_total, service_fee);
        self.service_fee_total += service_fee;

        self.next_quiz_id += 1;
        let total_questions = questions.len() as u16;

        let mut options_quantity = 0;

        let mut question_id: QuestionId = 0;
        for question in &questions {
            if let Some(question_options)  = all_question_options.get(question_id as usize) {
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
                &Question{
                    content: question.content.clone(),
                    hint: question.hint.clone(),
                    options_quantity,
                    kind: question.kind
                });

            question_id += 1;
        }

        let quiz = Quiz {
            owner_id: env::predecessor_account_id(),
            status: QuizStatus::Locked,
            total_questions,
            available_rewards_ids: unclaimed_rewards_ids,
            distributed_rewards_ids: Vec::new(),
            secret,
            success_hash,
            revealed_answers: None
        };
        self.quizzes.insert(&quiz_id, &quiz);

        quiz_id
    }

    pub fn activate_quiz(&mut self, quiz_id: QuizId, secret: Secret, success_hash: Hash){
        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            QuizChain::assert_current_user(&quiz.owner_id);
            assert_eq!(quiz.status, QuizStatus::Locked, "Quiz was already unlocked");

            quiz.secret = Some(secret);
            quiz.status = QuizStatus::InProgress;
            quiz.success_hash = Some(success_hash);
            self.quizzes.insert(&quiz_id, &quiz);
            self.active_quizzes.insert(&quiz_id);
        }
    }

    #[payable]
    pub fn reveal_answers(&mut self, quiz_id: QuizId, revealed_answers: Vec<RevealedAnswer>) {
        assert_one_yocto();

        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            QuizChain::assert_current_user(&quiz.owner_id);
            assert_eq!(quiz.status, QuizStatus::Finished, "Quiz is not finished");
            assert_eq!(quiz.total_questions, revealed_answers.len() as u16, "Illegal answers quantity");
            // todo add answers check

            quiz.revealed_answers = Some(revealed_answers);
            self.quizzes.insert(&quiz_id, &quiz);
        }
    }

    pub(crate) fn assert_current_user(owner_id: &AccountId){
        assert_eq!(*owner_id, env::predecessor_account_id(), "No access");
    }

    pub fn get_quiz(&self, quiz_id: QuizId) -> Option<QuizOutput> {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            Some(QuizOutput{
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
            })
        }
        else {
            None
        }
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
}