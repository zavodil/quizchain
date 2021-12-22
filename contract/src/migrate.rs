use crate::*;

#[near_bindgen]
impl QuizChain {

    #[private]
    #[init(ignore_state)]
    #[allow(dead_code)]
    pub fn migrate_1() -> Self {
        #[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
        #[serde(crate = "near_sdk::serde")]
        pub enum QuizStatusOld {
            Locked,
            InProgress,
            Finished
        }

        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct QuizOld {
            title: String,
            description: String,
            owner_id: AccountId,
            status: QuizStatusOld,
            total_questions: u16,
            available_rewards_ids: Vec<RewardId>,
            distributed_rewards_ids: Vec<RewardId>,
            secret: Option<Secret>,
            success_hash: Option<Hash>,
            revealed_answers: Option<Vec<RevealedAnswer>>,
        }

        #[derive(BorshDeserialize)]
        struct OldContract {
            active_quizzes: UnorderedSet<QuizId>,
            quizzes: LookupMap<QuizId, QuizOld>,
            questions: LookupMap<QuestionByQuiz, Question>,
            question_options: LookupMap<QuestionOptionByQuiz, QuestionOption>,
            rewards: LookupMap<RewardByQuiz, Reward>,
            games: LookupMap<QuizByUser, Game>,
            players: LookupMap<QuizId, UnorderedSet<AccountId>>,
            answers: LookupMap<AnswerByQuizByQuestionByUser, Answer>,
            next_quiz_id: QuizId,
            service_fee_total: Balance,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        let mut quizzes_new = LookupMap::new(StorageKey::QuizzesV1);
        let available_ids: Vec<QuizId> = [0].to_vec();
        for quiz_id in available_ids {
            if let Some(quiz) = old_contract.quizzes.get(&quiz_id) {
                quizzes_new.insert(&quiz_id,
                                   &Quiz {
                                       title: Some(quiz.title),
                                       description: Some(quiz.description),
                                       language: None,
                                       finality_type: QuizFinalityType::Direct,
                                       owner_id: quiz.owner_id,
                                       status: QuizStatus::InProgress,
                                       total_questions: quiz.total_questions,
                                       available_rewards_ids: quiz.available_rewards_ids,
                                       distributed_rewards_ids: quiz.distributed_rewards_ids,
                                       secret: quiz.secret,
                                       success_hash: quiz.success_hash,
                                       revealed_answers: quiz.revealed_answers,
                                       sponsor_account_id: None,
                                       funded_amount: None,
                                       restart_allowed: false,
                                       timestamp: None,
                                       token_account_id: Some(QuizChain::unwrap_token_id(&None))
                                   });
            }
        }

        Self {
            active_quizzes: old_contract.active_quizzes,
            quizzes: quizzes_new,

            questions: old_contract.questions,
            question_options: old_contract.question_options,
            rewards: old_contract.rewards,

            games: old_contract.games,
            players: old_contract.players,
            answers: old_contract.answers,

            next_quiz_id: old_contract.next_quiz_id,
            service_fees_total: LookupMap::new(StorageKey::ServiceFeesTotal),

            quiz_results: LookupMap::new(StorageKey::QuizResultsForDelayedFinality),
            whitelisted_tokens: LookupSet::new(StorageKey::WhitelistedTokens),

            quizzes_by_player_id: LookupMap::new(StorageKey::QuizzesByPlayer),
            quizzes_by_owner_id: LookupMap::new(StorageKey::QuizzesByOwner),
            quizzes_by_sponsor_id: LookupMap::new(StorageKey::QuizzesBySponsor),

            affiliates: LookupMap::new(StorageKey::Affiliates),
            total_affiliates: UnorderedMap::new(StorageKey::TotalAffiliates),
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate_2() -> Self {
        #[derive(BorshDeserialize)]
        struct OldContract {
            active_quizzes: UnorderedSet<QuizId>,
            quizzes: LookupMap<QuizId, Quiz>,

            questions: LookupMap<QuestionByQuiz, Question>,
            question_options: LookupMap<QuestionOptionByQuiz, QuestionOption>,
            rewards: LookupMap<RewardByQuiz, Reward>,

            games: LookupMap<QuizByUser, Game>,
            players: LookupMap<QuizId, UnorderedSet<AccountId>>,
            answers: LookupMap<AnswerByQuizByQuestionByUser, Answer>,

            next_quiz_id: QuizId,
            service_fees_total: LookupMap<TokenAccountId, Balance>,

            quiz_results: LookupMap<QuizResultByQuiz, Vec<AccountId>>,
            whitelisted_tokens: LookupSet<TokenAccountId>,

            quizzes_by_player_id: LookupMap<AccountId, Vec<QuizId>>,
            quizzes_by_owner_id: LookupMap<AccountId, Vec<QuizId>>,
            quizzes_by_sponsor_id: LookupMap<AccountId, Vec<QuizId>>
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        Self {
            active_quizzes: old_contract.active_quizzes,
            quizzes: old_contract.quizzes,

            questions: old_contract.questions,
            question_options: old_contract.question_options,
            rewards: old_contract.rewards,

            games: old_contract.games,
            players: old_contract.players,
            answers: old_contract.answers,

            next_quiz_id: old_contract.next_quiz_id,
            service_fees_total: old_contract.service_fees_total,

            quiz_results: old_contract.quiz_results,
            whitelisted_tokens: old_contract.whitelisted_tokens,

            quizzes_by_player_id: old_contract.quizzes_by_player_id,
            quizzes_by_owner_id: old_contract.quizzes_by_owner_id,
            quizzes_by_sponsor_id: old_contract.quizzes_by_sponsor_id,

            affiliates: LookupMap::new(StorageKey::Affiliates),
            total_affiliates: UnorderedMap::new(StorageKey::TotalAffiliates),
        }
    }
}
