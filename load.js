const CONTRACT_NAME = process.env.CONTRACT_NAME ||'dev-1632590154585-2196751'

const {NearView} = require('./near');
const AES = require('crypto-js/aes');


NearView(CONTRACT_NAME, "get_questions_by_quiz", {quiz_id: 1})
    .then((resp) => {
        resp.map(item => {
            console.log(`${item.question.content}: `);
            item.question_options.map(question_option=>console.log(`- ${question_option.content}`))
        })
        //console.log(resp);
    });
