let woj = require("./pkg/wasm_odessay_jps.js");
let map = Array(30 * 30).fill().map(x => Math.random() > 0.6 ? "1" : "0").join('');
map = `
00000000
00000000
00000000
00011111
00010000
01110000
00000000
00000000
`;
let x = 8;

let mapp = new Uint8Array(map.split('').filter(x => x == "0" || x == "1").map(x => Number(x)));
console.log(woj.a_star_jps(mapp, x, mapp.length / x, 7, 0, 7, 4, false));
