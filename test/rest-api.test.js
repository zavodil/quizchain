import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');

const alice = "grant.testnet";
const bob = "place.testnet";
const contract_id = process.env.CONTRACT_NAME;
const reward = 1;
const service_fee_ratio = 0.01;

const near = new contract(contract_id);

describe("Contract set", () => {
    test("Contract is not null " + contract_id, async () => {
        expect(contract_id).not.toBe(undefined)
    });

    test("Init contract", async () => {
        await near.call("new", {}, {account_id: contract_id});
    });

    test('Accounts has enough funds', async () => {
        const alice_wallet_balance = await near.accountNearBalance(alice, 5000);
        expect(alice_wallet_balance).toBeGreaterThan(20);

        const bob_wallet_balance = await near.accountNearBalance(alice, 5000);
        expect(bob_wallet_balance).toBeGreaterThan(20);
    });

});

let quiz_id = -1;
describe("Quiz", () => {
    test('Create Quiz', async () => {
        let active_quizzes_1 = await near.view("get_active_quizzes", {}, {});
        let active_quizzes_qty_1 = active_quizzes_1.length;

        quiz_id = await near.call("create_quiz",
            {
                title: "Test QUIZ",
                description: "Dummy text about the quiz",
                finality_type: "Direct",
                restart_allowed: true,
                questions: [
                    {"kind": "OneChoice", "content": "Какое сейчас время года?"},
                    {"kind": "MultipleChoice", "content": "Какие цифры четные"},
                    {"kind": "Text", "content": "Столица США", "hint": "Пишите на русском языке без опечаток"}
                ],
                all_question_options: [
                    [{"content": "Зима", "kind": "Text"}, {"content": "Весна", "kind": "Text"}, {
                        "content": "Осень",
                        "kind": "Text"
                    }],
                    [{"content": "2", "kind": "Text"}, {"content": "4", "kind": "Text"}, {
                        "content": "6",
                        "kind": "Text"
                    }],
                    []
                ],
                rewards:
                    [{"amount": utils.ConvertToNear(reward)}]
            }, {
                account_id: alice,
                tokens: utils.ConvertToNear(reward + reward * service_fee_ratio),
                log_errors: true,
                return_value: true
            });
        quiz_id = parseInt(quiz_id);
        expect(quiz_id).toBeGreaterThan(-1);

        let quiz = await near.view("get_quiz", {quiz_id}, {});
        expect(quiz.status).toBe("Locked");

        let activate_quiz = await near.call("activate_quiz", {
            quiz_id,
            secret: "77777",
            success_hash: "d35fee8b00d489a548f54b180c973c75b8c7b0c9483f5d01f1336c0ad1c701e9"
        }, {
            account_id: alice,
            log_errors: true
        });
        expect(activate_quiz.type).not.toBe('FunctionCallError');

        quiz = await near.view("get_quiz", {quiz_id}, {});
        expect(quiz.status).toBe("InProgress");

        let active_quizzes_2 = await near.view("get_active_quizzes", {}, {});
        let active_quizzes_qty_2 = active_quizzes_2.length;

        expect(active_quizzes_qty_2 - active_quizzes_qty_1).toBe(1);
    });

    test('Play Quiz', async () => {
        let start_game = await near.call("start_game", {
            quiz_id,
        }, {
            account_id: alice,
            log_errors: true
        });
        expect(start_game.type).not.toBe('FunctionCallError');

        let game_0 = await near.view("get_game", {quiz_id, account_id: alice}, {});
        console.log(game_0);
        expect(game_0.current_hash).toBe('816e2845d395e7703abac2dcbf9d54e39236fd39133362bf7ad3fce70dd7d78e');
        expect(game_0.answers_quantity).toBe(0);

        let send_answer_1 = await near.call("send_answer", {
            quiz_id,
            question_id: 0,
            question_option_ids: [2]
        }, {
            account_id: alice,
            log_errors: true
        });
        expect(send_answer_1.type).not.toBe('FunctionCallError');

        let game_1 = await near.view("get_game", {quiz_id, account_id: alice}, {});
        expect(game_1.current_hash).toBe('57266ce6dace2a2a7cd55253719eb523f75736aebf3d60647da13ec175ab938e');
        expect(game_1.answers_quantity).toBe(1);

        let send_answer_2 = await near.call("send_answer", {
            quiz_id,
            question_id: 1,
            question_option_ids: [0,1,2]
        }, {
            account_id: alice,
            log_errors: true
        });
        expect(send_answer_2.type).not.toBe('FunctionCallError');

        let game_2 = await near.view("get_game", {quiz_id, account_id: alice}, {});
        expect(game_2.current_hash).toBe('98625b0feb1c8313d69e50749a315da57a73b5d340c70dde1c8d86fdd5e5b5fb');
        expect(game_2.answers_quantity).toBe(2);

        let send_answer_3 = await near.call("send_answer", {
            quiz_id,
            question_id: 2,
            question_option_text: "Вашингтон"
        }, {
            account_id: alice,
            log_errors: true
        });
        expect(send_answer_3.type).not.toBe('FunctionCallError');

        let game_3 = await near.view("get_game", {quiz_id, account_id: alice}, {});
        expect(game_3.current_hash).toBe('d35fee8b00d489a548f54b180c973c75b8c7b0c9483f5d01f1336c0ad1c701e9');
        expect(game_3.answers_quantity).toBe(3);

        let quiz = await near.view("get_quiz", {quiz_id}, {});
        expect(quiz.distributed_rewards.length).toBeGreaterThan(0);
        expect(quiz.distributed_rewards[0].winner_account_id).toBe(alice);
    });
});


