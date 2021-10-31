use crate::*;


#[near_bindgen]
impl QuizChain {
    #[private]
    pub fn whitelist_token(&mut self, token_id: TokenAccountId) {
        self.whitelisted_tokens.insert(&token_id);
    }

    pub fn is_whitelisted_token(&self, token_id: TokenAccountId) -> bool {
        self.whitelisted_tokens.contains(&token_id)
    }

    pub(crate) fn assert_check_whitelisted_token(&self, token_id: &Option<TokenAccountId>) {
        if let Some(token_id) = token_id {
            assert!(self.whitelisted_tokens.contains(&token_id), "Token wasn't whitelisted");
        }
    }

    pub(crate) fn unwrap_token_id(token_id: &Option<TokenAccountId>) -> TokenAccountId {
        token_id.clone().unwrap_or_else(|| NEAR.to_string())
    }

    pub (crate) fn add_service_fees_total(&mut self, amount: Balance, token_account_id: &Option<TokenAccountId>){
        let token_id = QuizChain::unwrap_token_id(token_account_id);
        log!("{} of {} went to treasury", amount, token_id);
        let balance = self.service_fees_total.get(&token_id).unwrap_or(0);
        self.service_fees_total.insert(&token_id, &(balance + amount));
    }
}
