let woj = require("./pkg/wasm_odessay_jps.js");
let map = Array(30 * 30).fill().map(x => Math.random() > 0.6 ? "1" : "0").join('');
map = `
000000000
001000000
111111000
000000000
001111111
001000000
001011100
001100100
000001100
`;
let x = 9;

let mapp = new Uint8Array(map.split('').filter(x => x == "0" || x == "1").map(x => Number(x)));
console.log(woj.a_star_jps(mapp, x, mapp.length / x, 0, 0, 8, 8, true, true));
