use crate::*;
use near_sdk::{Gas, PromiseResult};
use std::convert::TryFrom;
use near_sdk::json_types::ValidAccountId;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_AFTER_FT_TRANSFER: Gas = 10_000_000_000_000;
const NO_DEPOSIT: Balance = 0;
const ONE_YOCTO: Balance = 1;

#[ext_contract(ext_self)]
pub trait ExtQuizChain {
    fn after_ft_withdraw(&mut self, account_id: AccountId, amount: WrappedBalance,
                         quiz_id: Option<QuizId>, reward_id: Option<RewardId>) -> bool;
}

#[near_bindgen]
impl QuizChain {

    pub(crate) fn withdraw(&mut self, recipient_account_id: AccountId, amount: Balance, token_account_id: Option<TokenAccountId>,
                           quiz_id: Option<QuizId>, reward_id: Option<RewardId>) -> Promise{
        let token_id_unwrapped = QuizChain::unwrap_token_id(&token_account_id);

        if token_id_unwrapped == NEAR {
            Promise::new(recipient_account_id).transfer(amount)
        } else {
            ext_fungible_token::ft_transfer(
                recipient_account_id.clone(),
                amount.into(),
                Some(format!("Withdraw: {} of {:?} from @{}", amount, token_id_unwrapped, env::current_account_id())),
                &token_id_unwrapped,
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER,
            )
                .then(ext_self::after_ft_withdraw(
                    recipient_account_id,
                    amount.into(),
                    quiz_id,
                    reward_id,
                    &env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_FOR_AFTER_FT_TRANSFER,
                ))
        }
    }

    #[private]
    pub fn after_ft_withdraw(
        &mut self,
        account_id: AccountId,
        amount: WrappedBalance,
        quiz_id: Option<QuizId>,
        reward_id: Option<RewardId>
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            if let Some(quiz_id_unwrapped) = quiz_id {
                if let Some(reward_id_unwrapped) = reward_id{
                    let index = QuizChain::get_reward_by_quiz(quiz_id_unwrapped, reward_id_unwrapped);
                    if let Some (reward) = self.rewards.get(&index){
                        self.rewards.insert(&index, &Reward {
                            amount: reward.amount + amount.0,
                            winner_account_id: reward.winner_account_id,
                            claimed: false
                        });
                        log!(
                            "FT withdraw for {} failed. Tokens to recharge: {}",
                            account_id,
                            amount.0
                        );
                    }
                }
            }
        }
        promise_success
    }

    pub fn after_ft_transfer_deposit(
        &mut self,
        account_id: AccountId,
        amount: WrappedBalance,
        token_account_id: TokenAccountId,
        quiz_id: Option<QuizId>,
        reward_id: Option<RewardId>
    ) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );

        let promise_success = is_promise_success();

        if !promise_success && quiz_id.is_some() && reward_id.is_some() {
            let quiz_id_unwrapped = quiz_id.unwrap();
            let reward_id_unwrapped = reward_id.unwrap();

                log!(
                "Token {} withdraw for user {} failed. Amount to return claim status: {}, quiz_id: {}, reward_id: {}",
                token_account_id,
                account_id,
                amount.0,
                quiz_id_unwrapped,
                reward_id_unwrapped
            );

                let reward_index = QuizChain::get_reward_by_quiz(quiz_id_unwrapped, reward_id_unwrapped);
                if let Some(mut reward) = self.rewards.get(&reward_index) {
                    reward.claimed = false;
                    self.rewards.insert(&reward_index, &reward);
                }

        }
        else {
            log!("Token {} withdraw for user {} failed. Recover unavailable", token_account_id, account_id);
        }

        promise_success
    }

    pub fn claim_reward(&mut self, quiz_id: QuizId) -> PromiseOrValue<bool> {
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            let user_rewards = self.get_user_reward_by_quiz(
                quiz_id, ValidAccountId::try_from(env::predecessor_account_id()).unwrap());
            if let Some(reward) = user_rewards {
                assert!(!reward.claimed, "Already claimed");
                return if let Some(winner_account_id) = reward.winner_account_id.clone() {
                    self.rewards.insert(&QuizChain::get_reward_by_quiz(quiz_id, reward.id), &Reward {
                        amount: reward.amount.0,
                        winner_account_id: reward.winner_account_id,
                        claimed: true
                    });
                    PromiseOrValue::Promise(self.withdraw(winner_account_id, reward.amount.0, quiz.token_account_id, Some(quiz_id), Some(reward.id)))
                } else {
                    PromiseOrValue::Value(false)
                }
            }
        }

        return PromiseOrValue::Value(false);
    }

    pub fn get_unclaimed_rewards_by_quiz(&self, quiz_id: QuizId) -> Vec<RewardOutput> {
        let mut rewards: Vec<RewardOutput> = Vec::new();
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            for reward_id in &quiz.available_rewards_ids {
                if let Some(reward) = self.rewards.get(&QuizChain::get_reward_by_quiz(quiz_id, *reward_id)) {
                    rewards.push(RewardOutput {
                        id: *reward_id,
                        amount: reward.amount.into(),
                        winner_account_id: reward.winner_account_id,
                        claimed: reward.claimed,
                    });
                }
            }
        }
        rewards
    }

    pub fn get_distributed_rewards_by_quiz(&self, quiz_id: QuizId) -> Vec<RewardOutput> {
        let mut rewards: Vec<RewardOutput> = Vec::new();
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            for reward_id in &quiz.distributed_rewards_ids {
                if let Some(reward) = self.rewards.get(&QuizChain::get_reward_by_quiz(quiz_id, *reward_id)) {
                    rewards.push(RewardOutput {
                        id: *reward_id,
                        amount: reward.amount.into(),
                        winner_account_id: reward.winner_account_id,
                        claimed: reward.claimed,
                    });
                }
            }
        }
        rewards
    }

    pub fn get_user_reward_by_quiz(&self, quiz_id: QuizId, account_id: ValidAccountId) -> Option<RewardOutput> {
        let user_id_value: AccountId = account_id.into();
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            for reward_id in &quiz.distributed_rewards_ids {
                if let Some(reward) = self.rewards.get(&QuizChain::get_reward_by_quiz(quiz_id, *reward_id)) {
                    if reward.winner_account_id.clone().unwrap() == user_id_value {
                        return Some(RewardOutput {
                            id: *reward_id,
                            amount: reward.amount.into(),
                            winner_account_id: reward.winner_account_id,
                            claimed: reward.claimed,
                        });
                    }
                }
            }
        }

        None
    }

    pub fn get_service_fee_total(&self, token_account_id: TokenAccountId) -> WrappedBalance {
        self.service_fees_total.get(&token_account_id).unwrap_or(0).into()
    }
}

fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}
