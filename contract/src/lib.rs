// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Timestamp, log, assert_one_yocto};
use near_sdk::collections::{LookupMap, UnorderedSet /* UnorderedMap , Vector*/ };
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::WrappedBalance;
use sha2::{Sha256, Digest};

pub use crate::quiz::*;
mod quiz;
mod internal;
mod rewards;
mod crypto;
mod game;

type QuizId = u64;
type QuestionId = u16;
type QuestionOptionId = u16;
type AnswerId = u16;
type RewardId = u16;
type Secret = String;
type Hash = String;


near_sdk::setup_alloc!();

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct QuizChain {
    active_quizzes: UnorderedSet<QuizId>,
    quizzes: LookupMap<QuizId, Quiz>,

    questions: LookupMap<QuestionByQuiz, Question>,
    question_options: LookupMap<QuestionOptionByQuiz, QuestionOption>,
    rewards: LookupMap<RewardByQuiz, Reward>,

    games: LookupMap<QuizByUser, Game>,
    answers: LookupMap<AnswerByQuizByQuestionByUser, Answer>,

    next_quiz_id: QuizId,
    service_fee_total: Balance
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Quiz {
    owner_id: AccountId,
    status: QuizStatus,

    total_questions: u16,

    available_rewards_ids: Vec<RewardId>,
    distributed_rewards_ids: Vec<RewardId>,

    secret: Option<Secret>,
    success_hash: Option<Hash>,

    revealed_answers: Option<Vec<RevealedAnswer>>
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
    current_hash: String
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    selected_option_ids: Option<Vec<QuestionOptionId>>,
    selected_text: Option<String>,
    timestamp: Timestamp
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
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
pub enum QuizStatus {
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
    Answers,
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
            answers: LookupMap::new(StorageKey::Answers),

            next_quiz_id: 0,
            service_fee_total: 0
        }
    }
}

#[near_bindgen]
impl QuizChain {

}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = QuizChain::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(
            "howdy".to_string(),
            contract.get_greeting("bob_near".to_string())
        );
    }

    #[test]
    fn get_default_greeting() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = QuizChain::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            "Hello".to_string(),
            contract.get_greeting("francis.near".to_string())
        );
    }
}
