let map_draw = document.getElementById("map-draw");
let map_mapX = document.getElementById("map-mapX");
let map_mapY = document.getElementById("map-mapY");
let map_cross = document.getElementById("map-cross");

let memory = [];

async function loadwasm() {
	await wasm_bindgen("pkg/wasm_odessay_jps_bg.wasm")
}

let mapX = 0,
	mapY = 0,
	stX, stY, edX, edY;

function mapScale() {
	let mX = Math.min(Math.floor(Math.max(Number(map_mapX.value), 1)), 1000);
	let mY = Math.min(Math.floor(Math.max(Number(map_mapY.value), 1)), 1000);
	memory = Array(mX * mY).fill(0);
	mapX = mX;
	mapY = mY;
	stX = 0;
	stY = 0;
	edX = mX - 1;
	edY = mY - 1;
	path = mapCalc();
	mapDraw();
	return true;
}

loadwasm().then(function() {
	mapScale();
	map_mapX.addEventListener("change", mapScale);
	map_mapY.addEventListener("change", mapScale);
	map_cross.addEventListener("change", function(event) {
		path = mapCalc();
		mapDraw();
	});
	window.addEventListener("resize", mapDraw);
}).catch((e) => {
	console.error(e);
	alert(e.message);
});

function mapDraw() {
	let scale = Math.min(
		(document.body.clientWidth - 40) / mapX,
		(document.body.clientHeight - 40) / mapY
	);
	map_draw.style.width = scale * mapX + "px";
	map_draw.style.height = scale * mapY + "px";
	map_draw.style.border = "1px white solid";
	map_draw.width = mapX * 30;
	map_draw.height = mapY * 30;
	let ct = map_draw.getContext("2d");
	ct.clearRect(0, 0, mapX * 30, mapY * 30);
	ct.fillStyle = "#000000";
	for (let y = 0; y < mapY; y++) {
		for (let x = 0; x < mapX; x++) {
			if (memory[y * mapX + x] === 1) {
				ct.fillRect(30 * x, 30 * y, 30, 30);
			}
		}
	}
	ct.fillStyle = "#d0d000";
	ct.strokeStyle = "#0000d0";
	ct.lineWidth = 3;
	for (let i = 0; i < path.length; i += 2) {
		ct.fillRect(30 * path[i], 30 * path[i + 1], 30, 30);
	}
	ct.fillStyle = "#ff0000";
	ct.fillRect(30 * stX, 30 * stY, 30, 30);
	ct.fillStyle = "#00d000";
	ct.fillRect(30 * edX, 30 * edY, 30, 30);
	for (let i = 0; i < path.length - 2; i += 2) {
		ct.beginPath();
		ct.moveTo(30 * path[i] + 15, 30 * path[i + 1] + 15);
		ct.lineTo(30 * path[i + 2] + 15, 30 * path[i + 3] + 15);
		ct.stroke();
	}
}

map_draw.addEventListener("touchstart", mapChangeStart);
map_draw.addEventListener("touchmove", mapChangeStep);
map_draw.addEventListener("mousedown", mapChangeStart);
map_draw.addEventListener("mousemove", mapChangeStep);
window.addEventListener("mouseleave", mapChangeStop);
map_draw.addEventListener("mouseup", mapChangeStop);

let changeMode = -1;

function mapChangeStart(event) {
	return mapChangeStep(event, true);
}

function mapChangeStep(event, isStart) {
	let rect = event.target.getBoundingClientRect();
	let isMouse = event.type.includes("mouse");
	let xd = ((isMouse ? event.clientX : event.targetTouches[0].clientX) - rect.left) / rect.width;
	let yd = ((isMouse ? event.clientY : event.targetTouches[0].clientY) - rect.top) / rect.height;
	if (0 <= xd && xd < 1 && 0 <= yd && yd < 1) {
		let xg = Math.floor(xd * mapX);
		let yg = Math.floor(yd * mapY);
		if (isStart) {
			if (xg === edX && yg === edY) {
				changeMode = 3;
			} else if (xg === stX && yg === stY) {
				changeMode = 2;
			} else {
				changeMode = memory[yg * mapX + xg] ^= 1;
			}
		} else {
			switch (changeMode) {
				case -1:
					return;
				case 2:
					if (memory[yg * mapX + xg] === 0 && !(xg === edX && yg === edY)) {
						stX = xg;
						stY = yg;
					}
					break;
				case 3:
					if (memory[yg * mapX + xg] === 0 && !(xg === stX && yg === stY)) {
						edX = xg;
						edY = yg;
					}
					break;
				default:
					if (!(xg === edX && yg === edY) && !(xg === stX && yg === stY)) {
						memory[yg * mapX + xg] = changeMode;
					}
			}
		}
		path = mapCalc();
		mapDraw();
		event.preventDefault();
	}
}

function mapChangeStop(event) {
	changeMode = -1;
}

function mapCalc() {
	let mode = map_cross.checked ? 2 : 1;
	let path = wasm_bindgen.a_star_jps(new Int8Array(memory), mapX, mapY, stX, stY, edX, edY, false);
	return Array.from(path);
}
