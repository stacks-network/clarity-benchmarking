const axios = require('axios');

// const NUM_REQUESTS = 1000;
const NUM_REQUESTS = 3;

var results = [];
for (var i = 0; i < NUM_REQUESTS; ++i) {
    const result = axios.get('http://localhost:20443/v2/accounts/SP1P72Z3704VMT3DMHPP2CB8TGQWGDBHD3RPR9GZS')
    // const result = axios.get('https://stacks-node-api.mainnet.stacks.co/v2/accounts/SP1P72Z3704VMT3DMHPP2CB8TGQWGDBHD3RPR9GZS')
    results.push(result)
}

console.log({
    results
})

var settled = 0;
while (settled < NUM_REQUESTS) {
    for (const promise of results) {
        promise.then(result => {
        settled += 1;
        console.log({
            settled,
            promise,
        })
        })
    }
}

process.exit()
