let webSocketUrls = [];

const register = async () => {
    const registerUrl = process.env.WS_HTTP_HOST + '/register';
    const gameId = parseInt(Math.floor(Math.random() * 100));
    const userId = parseInt(Math.floor(Math.random() * 100));

    const res = await fetch(registerUrl, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            game_id: gameId,
            user_id: userId,
        }),

    }).then(res => res.json());

    return res.url;
};

module.exports = {
    setWebSocketUrl: function (context, events, done) {
        webSocketUrls.push(context.vars.webSocketUrl);

        return done();
    },
    getWebSocketUrl: function (context, events, done) {
        if (webSocketUrls.length > 0) {
            context.vars.webSocketUrl = webSocketUrl.shift();
        }

        return done();
    },
    connect: async function (params, context, done) {
        const url = await register();
        const webSocketUrl = process.env.WS_HOST + url;
        
        params.target = webSocketUrl;

        return done();
    },
    fillParameters: async function (params, context, events, done) {
        params.json.game_id = parseInt(Math.floor(Math.random() * 100));
        params.json.user_id = parseInt(Math.floor(Math.random() * 100));

        return done();
    }
}