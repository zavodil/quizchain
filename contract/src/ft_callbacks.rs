use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferArgs {
    pub operation: String,
    pub quiz_owner_id: ValidAccountId,
    pub title: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub finality_type: QuizFinalityType,
    pub questions: Vec<QuestionInput>,
    pub all_question_options: Vec<Vec<QuestionOption>>,
    pub rewards: Vec<RewardInput>,
    pub secret: Option<String>,
    pub success_hash: Option<String>,
    pub restart_allowed: bool,
}

trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: ValidAccountId, amount: WrappedBalance, msg: String) -> PromiseOrValue<WrappedBalance>;
}

#[near_bindgen]
impl FungibleTokenReceiver for QuizChain {
    fn ft_on_transfer(&mut self, sender_id: ValidAccountId, amount: WrappedBalance, msg: String) -> PromiseOrValue<WrappedBalance> {
        let token_account_id: Option<TokenAccountId> = Some(env::predecessor_account_id());
        self.assert_check_whitelisted_token(&token_account_id);

        let TransferArgs {
            operation,
            quiz_owner_id,
            title,
            description,
            language,
            finality_type,
            questions,
            all_question_options,
            rewards,
            secret,
            success_hash,
            restart_allowed
        } = near_sdk::serde_json::from_str(&msg).expect("Invalid TransferArgs");

        let quiz_owner_value: AccountId = quiz_owner_id.into();

        if operation == "create_quiz_for_account" {
            self.create_quiz_for_account_internal(
                sender_id.into(),
                quiz_owner_value.clone(),
                amount.0,
                token_account_id);
        } else if operation == "create_quiz" {
            if let Some(secret_unwrapped) = secret {
                log!("create and activate");
                self.create_quiz_and_activate_internal(quiz_owner_value.clone(),
                                          title,
                                          description,
                                          language,
                                          finality_type,
                                          questions,
                                          all_question_options,
                                          rewards,
                                          secret_unwrapped,
                                          success_hash.clone(),
                                          restart_allowed,
                                          amount.0,
                                          token_account_id);
            }
            else {
                self.create_quiz_internal(quiz_owner_value.clone(),
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
                                          amount.0,
                                          token_account_id);
            }
        } else {
            panic!("Unknown operation");
        }

        PromiseOrValue::Value(WrappedBalance::from(0))
    }
}
