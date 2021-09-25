use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuizOutput {
    owner_id: AccountId,
    status: Status,
    total_questions: u16,
    unclaimed_rewards_ids: Vec<RewardId>,
    secret: Option<String>,
    success_hash: Option<String>,
    questions: Vec<QuestionOutput>,
    rewards: Vec<RewardOutput>,
}


// todo pay
#[near_bindgen]
impl QuizChain {
    pub fn create_quiz(&mut self,
                       questions: Vec<QuestionInput>,
                       all_question_options: Vec<Vec<QuestionOption>>,
                       rewards: Vec<RewardInput>,
                       secret: Option<String>,
                       success_hash: Option<String>) -> QuizId {
        assert_eq!(questions.len(), all_question_options.len(), "Questions and question options not matched");
        assert!(questions.len() > 0, "Data not found");

        let total_questions = questions.len() as u16;
        let mut unclaimed_rewards_ids = Vec::new();

        let quiz_id = self.next_quiz_id;
        self.next_quiz_id += 1;
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
                    options_quantity
                });

            question_id += 1;
        }

        let mut reward_id: RewardId = 0;
        for reward in &rewards {
            self.rewards.insert(
                &QuizChain::get_reward_by_quiz(quiz_id, reward_id),
                &Reward{
                    amount: reward.amount.0,
                    claimed_by: None
                });
            unclaimed_rewards_ids.push(reward_id);

            reward_id += 1;
        }

        let quiz = Quiz {
            owner_id: env::predecessor_account_id(),
            status: Status::Locked,
            total_questions,
            unclaimed_rewards_ids,
            secret,
            success_hash,
        };
        self.quizzes.insert(&quiz_id, &quiz);

        quiz_id
    }

    pub fn activate_quiz(&mut self, quiz_id: QuizId, secret: Secret, success_hash: Hash){
        if let Some(mut quiz) = self.quizzes.get(&quiz_id) {
            QuizChain::assert_current_user(&quiz.owner_id);
            assert_eq!(quiz.status, Status::Locked, "Quiz was already unlocked");

            quiz.secret = Some(secret);
            quiz.status = Status::InProgress;
            quiz.success_hash = Some(success_hash);
            self.quizzes.insert(&quiz_id, &quiz);
            self.active_quizzes.insert(&quiz_id);
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
                unclaimed_rewards_ids: quiz.unclaimed_rewards_ids,
                secret: quiz.secret,
                success_hash: quiz.success_hash,
                questions: self.get_questions_by_quiz(quiz_id),
                rewards: self.get_rewards_by_quiz(quiz_id)
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