"use strict";

import {blake3} from "https://cdn.jsdelivr.net/npm/@noble/hashes@2.0.0/blake3.js/+esm"

self.addEventListener("message", async (event) => {
    const {challenge, difficultyBits, expiresAt} = await event.data;
    let nonce = 0n;

    while (true) {
        let tempHash = blake3(new TextEncoder().encode(`${nonce}${challenge}${expiresAt}`))
            .reduce((acc, byte, i) => acc | (BigInt(byte) << (8n * BigInt(i))), 0n);
        let trailingZeros = 0;
        while ((tempHash & 1n) === 0n && tempHash !== 0n) {
            tempHash >>= 1n;
            trailingZeros++;
        }
        if (trailingZeros >= difficultyBits) {
            self.postMessage({ nonce: nonce.toString() });
            return;
        }
        nonce++;
    }
})