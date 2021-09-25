use crate::*;

#[near_bindgen]
impl QuizChain {
    pub fn get_rewards_by_quiz(&self, quiz_id: QuizId) -> Vec<RewardOutput> {
        let mut rewards: Vec<RewardOutput> = Vec::new();
        if let Some(quiz) = self.quizzes.get(&quiz_id) {
            for reward_id in &quiz.unclaimed_rewards_ids {
                if let Some(reward) = self.rewards.get(&QuizChain::get_reward_by_quiz(quiz_id, *reward_id)) {
                    rewards.push(RewardOutput {
                        id: *reward_id,
                        amount: reward.amount.into(),
                        claimed_by: reward.claimed_by,
                    });
                }
            }
        }
        rewards
    }
}