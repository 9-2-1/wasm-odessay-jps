let woj = require("./pkg/wasm_odessay_jps.js");
let map = Array(30 * 30).fill().map(x => Math.random() > 0.6 ? "1" : "0").join('');
map = `
00000000
00000000
00100000
00100000
00101110
01000000
00000000
`;
let x = 8;

let mapp = new Uint8Array(map.split('').filter(x => x == "0" || x == "1").map(x => Number(x)));
console.log(woj.a_star_jps(mapp, x, mapp.length / x, 0, 0, 7, 6, false));
