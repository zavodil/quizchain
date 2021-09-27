use std::convert::TryFrom;
use near_sdk::{Promise, PromiseOrValue};
use near_sdk::json_types::ValidAccountId;

use crate::*;

#[near_bindgen]
impl QuizChain {
    pub fn claim_reward(&mut self, quiz_id: QuizId) -> PromiseOrValue<bool> {
        let user_rewards = self.get_user_reward_by_quiz(
            quiz_id,ValidAccountId::try_from(env::predecessor_account_id()).unwrap());
        if let Some(reward) = user_rewards {
            assert_eq!(reward.claimed, false, "Already claimed");
            return if let Some(winner_account_id) = reward.winner_account_id.clone() {
                self.rewards.insert(&QuizChain::get_reward_by_quiz(quiz_id, reward.id), &Reward {
                    amount: reward.amount.0,
                    winner_account_id: reward.winner_account_id,
                    claimed: true
                });
                log!("Congratulations! {} yNEAR claimed", reward.amount.0);
                // TODO add transfer().then()...
                PromiseOrValue::Promise(Promise::new(winner_account_id).transfer(reward.amount.0))
            } else {
                PromiseOrValue::Value(false)
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

    pub fn get_service_fee_total(&self) -> WrappedBalance {
        self.service_fee_total.into()
    }
}