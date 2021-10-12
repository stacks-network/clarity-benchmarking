const axios = require('axios');
const process = require('process');

const arg = process.argv[2];
console.log({arg});
const size = parseInt(arg);

async function RunTests(NUM_REQUESTS) {
    const startTime = new Date().getTime();
    var results = [];
    for (var i = 0; i < NUM_REQUESTS; ++i) {
        const result = axios.get('http://localhost:20443/v2/accounts/SP1P72Z3704VMT3DMHPP2CB8TGQWGDBHD3RPR9GZS')
        // const result = axios.get('https://stacks-node-api.mainnet.stacks.co/v2/accounts/SP1P72Z3704VMT3DMHPP2CB8TGQWGDBHD3RPR9GZS')
        results.push(result)
    }

    var settled = 0;
    for (const promise of results) {
        const result = await promise;
    }

    const endTime = new Date().getTime();

    const diffTime = endTime - startTime;

    console.log({
            NUM_REQUESTS,
        diffTime,
    })
}

RunTests(size);
