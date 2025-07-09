const canvas = document.querySelector('canvas');
const context = canvas.getContext('2d');

let token = new URLSearchParams(window.location.search).get('token');

if (token) {
	localStorage.setItem('token', token);
} else {
	token = localStorage.getItem('token');
}

const peers = {};

const socket = io({
	auth: {
		token
	},
});

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

const draw_peers = (context, peers) => {
	context.clearRect(0, 0, context.canvas.width, context.canvas.height);

	for (const id in peers) {
		const peer = peers[id];

		if (peer.position) {
			context.beginPath();
			context.arc(peer.position.x, peer.position.y, 1, 0, Math.PI * 2);
			context.fillStyle = peer.color;
			context.fill();
		}
	}
};
