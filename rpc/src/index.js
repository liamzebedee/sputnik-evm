const express = require("express");
const bodyParser = require("body-parser");
const { JSONRPCServer } = require("json-rpc-2.0");
const { execSync } = require('child_process')
const { resolve, join } = require('path')
const tmp = require('tmp');
const { readFileSync } = require('fs');

// temporary path.
const { randomUUID } = require('crypto');
const os = require("os");
const getTmpFilePath = () => {
    const tempDir = os.tmpdir();
    return join(tempDir, '/', randomUUID())
}

const server = new JSONRPCServer();


// Handlers.
// 

const binary = process.env.DEV ? 'RUST_BACKTRACE=1 cargo run --' : 'cargo run --'
const command = `${binary} --db-path ./chain.sqlite`

const SPUTNIK_EXECUTOR_PATH = resolve(process.env.SPUTNIK_EXECUTOR_PATH)


server.addMethod("eth_call", (params) => {
    // Handle params as an array.
    let body = {}
    if(params.length) {
        body = params[0]
    } else {
        body = params
    }

    // TODO.
    delete body.accessList
    delete body.type

    // temp file to store output.
    const tempOutputFile = getTmpFilePath();

    // convert params into a string
    const sanitized = `'${JSON.stringify(body)}'`
    execSync(
        `${command} --data ${sanitized} --output-file ${tempOutputFile}`,
        {
            cwd: resolve(SPUTNIK_EXECUTOR_PATH)
        }
    )

    // read the temp file.
    let buf = readFileSync(tempOutputFile, { encoding: 'hex' })
    return '0x'+buf
});

server.addMethod("eth_sendTransaction", (params) => {
    // convert params into a string
    console.log(params)
    const sanitized = `'${JSON.stringify(params)}'`
    execSync(
        `${command} --data ${sanitized} --write`,
        {
            cwd: resolve(SPUTNIK_EXECUTOR_PATH)
        }
    )
});

server.addMethod("eth_sendRawTransaction", (params) => {
    // convert params into a string
    console.log(params)
    // const sanitized = `'${JSON.stringify(params)}'`
    // execSync(
    //     `${command} --data ${sanitized} --write`,
    //     {
    //         cwd: resolve(SPUTNIK_EXECUTOR_PATH)
    //     }
    // )
});

server.addMethod('eth_getTransactionReceipt', (params) => {

})

server.addMethod('eth_getTransactionCount', (params) => {
    return '0x1'
})

server.addMethod('eth_chainId', (params) => {
    return '0x1A4' // 420
})

server.addMethod('eth_gasPrice', (params) => {
    return '0x1'
})

// Stubbed methods.
const stubbed = 'eth_gasPrice eth_blockNumber eth_getBalance eth_getTransactionCount eth_getTransactionReceipt'


// App.
//

const app = express();
app.use(bodyParser.json());

app.post("/", (req, res) => {
    const jsonRPCRequest = req.body;

    server.receive(jsonRPCRequest).then((jsonRPCResponse) => {
        if (jsonRPCResponse) {
            res.json(jsonRPCResponse);
        } else {
            // If response is absent, it was a JSON-RPC notification method.
            // Respond with no content status (204).
            res.sendStatus(204);
        }
    });
});

const PORT = process.env.PORT || 8549

console.log(`Listening on http://localhost:${PORT}`)
console.log(`SPUTNIK_EXECUTOR_PATH = ${SPUTNIK_EXECUTOR_PATH}`)
app.listen(PORT);