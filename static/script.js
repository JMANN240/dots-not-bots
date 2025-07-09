const canvas = document.querySelector('canvas');
const context = canvas.getContext('2d');

function getCookie(cname) {
	let name = cname + "=";
	let decodedCookie = decodeURIComponent(document.cookie);
	let ca = decodedCookie.split(';');
	for (let i = 0; i < ca.length; i++) {
		let c = ca[i];
		while (c.charAt(0) == ' ') {
			c = c.substring(1);
		}
		if (c.indexOf(name) == 0) {
			return c.substring(name.length, c.length);
		}
	}
	return null;
}

const token = getCookie('token')

const peers = {};

const socket = io({
	auth: {
		token,
	},
});

const draw_peers = (context, peers) => {
	context.clearRect(0, 0, context.canvas.width, context.canvas.height);

	for (const id in peers) {
		const peer = peers[id];

		if (peer.position) {
			context.beginPath();
			context.arc(peer.position.x, peer.position.y, 3, 0, Math.PI * 2);
			context.fillStyle = peer.color;
			context.fill();
		}
	}
};

socket.on('data', (data) => {
	console.log("<- data", data);
	peers[data.id] = data;
	draw_peers(context, peers);
});

socket.on('disconnected', (id) => {
	console.log("<- disconnected", id);
	delete peers[id];
	draw_peers(context, peers);
});

if (token) {
	canvas.addEventListener('click', (e) => {
		const position = {
			x: e.offsetX,
			y: e.offsetY,
		};
		console.log("-> position", position);
		socket.emit('position', position)
	});
}
