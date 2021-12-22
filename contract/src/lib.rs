use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, ext_contract, AccountId, Balance, BorshStorageKey, PanicOnDefault,
               PromiseOrValue, Promise, Timestamp, log, assert_one_yocto};
use near_sdk::collections::{LookupMap, UnorderedSet, LookupSet, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{ValidAccountId, WrappedBalance};
use sha2::{Sha256, Digest};

pub use crate::quiz::*;
mod quiz;
mod internal;
mod rewards;
mod game;
mod migrate;
mod ft;
mod ft_callbacks;

type QuizId = u64;
type QuestionId = u16;
type QuestionOptionId = u16;
type AnswerId = u16;
type RewardId = u16;
type Secret = String;
type Hash = String;
type TokenAccountId = AccountId;

const NEAR: &str = "near";

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct QuizChain {
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
    quizzes_by_sponsor_id: LookupMap<AccountId, Vec<QuizId>>,

    affiliates: LookupMap<QuizId, UnorderedMap<AccountId, u64>>,
    total_affiliates: UnorderedMap<AccountId, u64>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Quiz {
    title: Option<String>,
    description: Option<String>,
    language: Option<String>,
    finality_type: QuizFinalityType,

    owner_id: AccountId,
    status: QuizStatus,

    total_questions: u16,

    available_rewards_ids: Vec<RewardId>,
    distributed_rewards_ids: Vec<RewardId>,

    secret: Option<Secret>,
    success_hash: Option<Hash>,

    revealed_answers: Option<Vec<RevealedAnswer>>,
    sponsor_account_id: Option<AccountId>,
    funded_amount: Option<Balance>,
    restart_allowed: bool,
    timestamp: Option<Timestamp>,
    token_account_id: Option<TokenAccountId>,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Question {
    kind: QuestionKind,
    content: String,
    hint: Option<String>,
    options_quantity: u16
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuestionOption {
    content: String,
    kind: QuestionOptionKind
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct QuestionInput {
    kind: QuestionKind,
    content: String,
    hint: Option<String>,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardInput {
    amount: WrappedBalance,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardOutput {
    id: RewardId,
    amount: WrappedBalance,
    winner_account_id: Option<AccountId>,
    claimed: bool
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuestionOutput {
    id: QuestionId,
    question: Question,
    question_options: Vec<QuestionOptionOutput>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuestionOptionOutput {
    id: QuestionId,
    content: String,
    kind: QuestionOptionKind,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AffiliatesOutput {
    account_id: AccountId,
    affiliates: u64
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuizByUser {
    quiz_id: QuizId,
    account_id: AccountId
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AnswerByQuizByQuestionByUser {
    quiz_id: QuizId,
    question_id: QuestionId,
    account_id: AccountId
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuestionByQuiz {
    quiz_id: QuizId,
    question_id: QuestionId
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuizResultByQuiz {
    quiz_id: QuizId,
    hash: Hash
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QuestionOptionByQuiz {
    quiz_id: QuizId,
    question_id: QuestionId,
    question_option_id: QuestionOptionId
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardByQuiz {
    quiz_id: QuizId,
    reward_id: RewardId
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Game {
    answers_quantity: u16,
    current_hash: Hash
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    selected_option_ids: Option<Vec<QuestionOptionId>>,
    selected_text: Option<String>,
    timestamp: Timestamp
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RevealedAnswer {
    selected_option_ids: Option<Vec<QuestionOptionId>>,
    selected_text: Option<String>
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Reward {
    amount: Balance,
    winner_account_id: Option<AccountId>,
    claimed: bool
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum QuizFinalityType {
    Direct,
    DelayedReveal
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum QuizStatus {
    Funded,
    Locked,
    InProgress,
    Finished
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum QuestionKind {
    OneChoice,
    MultipleChoice,
    Text
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum QuestionOptionKind {
    Text,
    Image,
    Html
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Quizzes,
    ActiveQuizzes,

    Questions,
    QuestionOptions,
    Rewards,

    Games,
    Players,
    Answers,

    QuizzesV1,
    QuizResultsForDelayedFinality,
    WhitelistedTokens,
    ServiceFeesTotal,

    QuizzesByPlayer,
    QuizzesByOwner,
    QuizzesBySponsor,

    Affiliates,
    AffiliatesByQuiz { quiz_id: u64 },
    TotalAffiliates,
}

#[near_bindgen]
impl QuizChain {
    #[init]
    pub fn new() -> Self {
        Self {
            quizzes: LookupMap::new(StorageKey::Quizzes),
            active_quizzes: UnorderedSet::new(StorageKey::ActiveQuizzes),

            questions: LookupMap::new(StorageKey::Questions),
            question_options: LookupMap::new(StorageKey::QuestionOptions),
            rewards: LookupMap::new(StorageKey::Rewards),

            games: LookupMap::new(StorageKey::Games),
            players: LookupMap::new(StorageKey::Players),
            answers: LookupMap::new(StorageKey::Answers),

            next_quiz_id: 0,
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
}
