const AES = require('crypto-js/aes');
const CryptoJS = require('crypto-js');

const secret = '77777';
let m1 = getHash(secret);
console.log(m1);
let m2 = getHash(m1 + 'осень');
console.log(m2);
let m3 = getHash(m2 + '2' + '4' + '6');
console.log(m3);
let m4 = getHash(m3 + 'вашингтон');
console.log(m4);

/*
816e2845d395e7703abac2dcbf9d54e39236fd39133362bf7ad3fce70dd7d78e
ca9ee4cafb3654a11c25936a41b045c687c27d2a30f093e447484164b0786f22
ca9ee4cafb3654a11c25936a41b045c687c27d2a30f093e447484164b0786f22o22
4fa4dce7e51c2ef58584836913ea30ad0ee469895e1ba15eaae4cba77e762481*/



/*
let m1 = 'e1d38d0b67272794e4d69f93d962f2fa5a34fa12808ab87cdc0d8c5c4337cbe1';
let m2 = getHash(m1.concat('o22'));
console.log(m2);

let m3 = getHash('e1d38d0b67272794e4d69f93d962f2fa5a34fa12808ab87cdc0d8c5c4337cbe1o22');
console.log(m3);

 */

//df2fa7d51b201ed2b599688097d7447283631ee5f82f06923695b6ed92af37d4

function getHash(text){
    let hash   = CryptoJS.SHA256(text);
    //let buffer = Buffer.from(hash.toString(CryptoJS.enc.Hex), 'hex');
    //let array  = new Uint8Array(buffer);
    //console.log(buffer);
    return hash.toString(CryptoJS.enc.Hex);
    //return buffer;
}

/*

let secret = "123123123123dfsdfsfs";

let enc = encrypt("123 2fwfew f efefe fefe few fewfe wefefew fewfew", secret);
console.log(enc);

function encrypt(text = '', key = ''){
    const message = AES.encrypt(text, key);
    return message.toString();
}

 */