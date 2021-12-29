const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const BN = require('bn.js');
const Decimal = require('decimal.js');
const khala_typedefs = require('@phala/typedefs').khalaDev;

const bnh64bits = new BN('FFFFFFFFFFFFFFFF0000000000000000', 16);
const bnl64bits = new BN('FFFFFFFFFFFFFFFF', 16);

async function getMiningPayout(api, hash) {
    return new Promise(async (resolve) => {
        api.query.system.events.at(hash, (events) => {
            let totalRewards = new Decimal(0.0);

            for (const record of events) {
                const { event, phase } = record;

                if (event.section === 'phalaMining' && event.method === 'MinerSettled') {
                    // data[0]: miner,
                    // data[1]: v,
                    // data[2]: payout,
                    let payout = event.data[2];
                    let a = new BN(payout.toString()).and(bnh64bits).shrn(64);
                    let b = new BN(payout.toString()).and(bnl64bits);
                    let strPayout = a.toString() + '.' + b.toString();
                    totalRewards = totalRewards.add(new Decimal(strPayout));
                }
            };
            
            resolve(totalRewards.toString());
        });
    });
}

async function main() { 
    const provider = new WsProvider('wss://khala.api.onfinality.io/public-ws');
    const api = await ApiPromise.create({
        provider: provider,
        types: {
            ...khala_typedefs,
            ...{
                Lottery: {
                    _enum: {
                        SignedTx: {
                            roundId: "u32",
                            tokenId: "Vec<u8>",
                            tx: "Vec<u8>"
                        },
                        BtcAddresses: {
                            addressSet: "Vec<Vec<u8>>"
                        }
                    }
                },
                BalancesTransfer: {
                    dest: "AccountId",
                    amount: "u128"
                },
                WorkerPinkReport: {
                    _enum: {
                        PinkInstantiated: {
                            id: "[u8; 32]",
                            groupId: "[u8; 32]",
                            owner: "AccountId",
                            pubkey: "EcdhPublicKey"
                        }
                    }
                },
                KittyTransfer: {
                    dest: "AccountId",
                    kittyId: "Vec<u8>"
                },
                WorkerEvent: {
                    _enum: {
                        Registered: "(WorkerInfo)",
                        BenchStart: {
                            duration: "u32"
                        },
                        BenchScore: "(u32)",
                        MiningStart: {
                            sessionId: "u32",
                            initV: "U64F64Bits",
                            initP: "u32"
                        },
                        MiningStop: null,
                        MiningStop: null,
                        MiningStop: null
                    }
                },
                WorkerEventWithKey: {
                    pubkey: "WorkerPublicKey",
                    event: "WorkerEvent"
                },
                SystemEvent: {
                    _enum: {
                        WorkerEvent: "(WorkerEventWithKey)",
                        HeartbeatChallenge: "(HeartbeatChallenge)"
                    }
                },
                MiningReportEvent: {
                    _enum: {
                        sessionId: "u32",
                        challengeBlock: "u32",
                        challengeTime: "u64",
                        iterations: "u64"
                    }
                },
                SettleInfo: {
                    pubkey: "WorkerPublicKey",
                    v: "U64F64Bits",
                    payout: "U64F64Bits",
                    treasury: "U64F64Bits"
                },
                MiningInfoUpdateEvent: {
                    blockNumber: "u64",
                    timestampMs: "u64",
                    offline: "Vec<WorkerPublicKey>",
                    recoveredToOnline: "Vec<WorkerPublicKey>",
                    settle: "Vec<SettleInfo>"
                }
            }
        }
    });

    const unsubscribe = await api.rpc.chain.subscribeFinalizedHeads(async (header) => {
        const payout = await getMiningPayout(api, header.hash);
        console.log(`Total mining payout of block ${header.number} is: ${payout}`);
    });

    const exitHandler = (options, exitCode) => {
        unsubscribe();
        if (options.cleanup) console.log('do cleanup');
        if (exitCode || exitCode === 0) console.log(exitCode);
        if (options.exit) process.exit();
    }

    process.stdin.resume();//so the program will not close instantly

    // Following copy from: https://stackoverflow.com/questions/14031763/doing-a-cleanup-action-just-before-node-js-exits/14032965#14032965
    // do something when app is closing
    process.on('exit', exitHandler.bind(null,{cleanup:true}));
    //catches ctrl+c event
    process.on('SIGINT', exitHandler.bind(null, {exit:true}));
    // catches "kill pid" (for example: nodemon restart)
    process.on('SIGUSR1', exitHandler.bind(null, {exit:true}));
    process.on('SIGUSR2', exitHandler.bind(null, {exit:true}));
    //catches uncaught exceptions
    process.on('uncaughtException', exitHandler.bind(null, {exit:true}));
}

main();