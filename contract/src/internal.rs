use crate::*;

#[near_bindgen]
impl QuizChain {
    pub(crate) fn get_question_by_quiz(quiz_id: QuizId, question_id: QuestionId) -> QuestionByQuiz{
        QuestionByQuiz{quiz_id, question_id}
    }

    pub(crate) fn get_question_option_by_quiz(quiz_id: QuizId, question_id: QuestionId, question_option_id: QuestionOptionId) -> QuestionOptionByQuiz{
        QuestionOptionByQuiz { quiz_id, question_id, question_option_id }
    }

    pub(crate) fn get_reward_by_quiz(quiz_id: QuizId, reward_id: RewardId) -> RewardByQuiz {
        RewardByQuiz { quiz_id, reward_id }
    }

    pub(crate) fn get_quiz_by_user(quiz_id: QuizId, account_id: AccountId) -> QuizByUser {
        QuizByUser { quiz_id, account_id }
    }

    pub(crate) fn get_answer_by_quiz_by_question(quiz_id: QuizId, question_id: QuestionId, account_id: AccountId) -> AnswerByQuizByQuestionByUser {
        AnswerByQuizByQuestionByUser { quiz_id, question_id, account_id }
    }
}