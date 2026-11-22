# KASCADE post-quantum wallets and transactions

November 22, 2026. Research for a post-launch upgrade. Launch and the Phase 6 Simnet preview proceed independently. This report proposes experiments and makes no protocol selection.

## Existing exposure and candidate signatures

The current transparent `PubKey` and `PubKeyECDSA` address payloads carry 32-byte x-only and 33-byte compressed secp256k1 public keys. `ScriptHash` uses a 32-byte script commitment. A commitment can delay public-key exposure until spending when the underlying policy and keys are kept private. Reused keys, exposed xpubs, and legacy spends in flight remain relevant to migration. A transparent signature change leaves Orchard-derived spend and binding authorization, note encryption, and Halo 2 proofs as a separate research track. ML-KEM is a key-establishment mechanism. [KASCADE addresses](../../crypto/addresses/src/lib.rs) · [KASCADE script templates](../../crypto/txscript/src/standard.rs) · [Orchard specification](https://zips.z.cash/zip-0224) · [NIST FIPS 203](https://csrc.nist.gov/pubs/fips/203/final)

| Suite | NIST category | Raw public key | Raw signature | Key + signature |
| --- | ---: | ---: | ---: | ---: |
| Current x-only Schnorr | — | 32 B | 64 B | 96 B |
| ML-DSA-44 | 2 | 1,312 B | 2,420 B | 3,732 B |
| ML-DSA-65 | 3 | 1,952 B | 3,309 B | 5,261 B |
| ML-DSA-87 | 5 | 2,592 B | 4,627 B | 7,219 B |
| SLH-DSA-SHA2-128s | 1 | 32 B | 7,856 B | 7,888 B |
| Falcon-padded-512 reference | 1 | 897 B | 666 B | 1,563 B |

Totals exclude framing, policy commitments, and other transaction fields. The current Schnorr key normally sits in the locking output. A new committed-key output shifts the revealed key to the spend. A hybrid witness revealing both keys costs 3,828 B with ML-DSA-44 or 5,357 B with ML-DSA-65 before framing. Compare those two ML-DSA profiles first. Include SLH-DSA as a hash-based comparator. Track FN-DSA after FIPS 206 is final and a suitable implementation has received side-channel review. The Falcon row describes a reference encoding. [BIP 340](https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki) · [FIPS 204 Table 2](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf) · [FIPS 205 Table 2](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.205.pdf) · [OQS Falcon](https://openquantumsafe.org/liboqs/algorithms/sig/falcon.html) · [NIST standardization](https://csrc.nist.gov/Projects/post-quantum-cryptography/post-quantum-cryptography-standardization)

## Addresses, authorization, and migration

Prototype a versioned network-specific address carrying a fixed-size hash of a canonical authorization policy. Keep raw public keys out of receive QR codes. Compare 256-bit and wider commitments against the threat model. Define the exact signed bytes, version, network, purpose, algorithm, and sighash rules, including prevouts, amounts and scripts. A hybrid option should require both signatures over identical authorization semantics with independent keys. Test exact encoding, cross-network replay rejection, and invalid or missing signatures.

Preserve legacy restoration and account scans. Use a separately versioned post-quantum account, with independent seed material or reviewed domain-separated hardened derivation. FIPS 204 permits regeneration from a 32-byte seed. Existing secp256k1 xpubs cannot derive these keys. Evaluate public descriptors that carry commitments or pre-generated receive keys for watch-only wallets. Test backing up and restoring both wallet generations, watch-only receive, and replacement of hardware-bound signers. After an approved upgrade and wallet/custodian support, move funds with ordinary legacy-authorized sweeps while legacy authorization is still reliable. Prioritize reused and already exposed keys in migration guidance. Legacy sunsets and dormant funds require distinct policy decisions. [NIST FIPS 204](https://csrc.nist.gov/pubs/fips/204/final) · [NIST migration guidance](https://csrc.nist.gov/pubs/ir/8547/final) · [BIP 361 draft](https://github.com/bitcoin/bips/blob/master/bip-0361.mediawiki)

## Verification and mobile

One published AArch64 benchmark using liboqs 0.16.0 measured typical valid ML-DSA-44 verification at about 59 µs on its fast core and 188–192 µs on its slower core. Its SHAKE-128s SLH-DSA comparator took roughly 0.92 ms and 3.02–3.13 ms. Treat these as platform-specific baselines. Measure transaction decoding, invalid and valid signature paths, rechecks, fees, relay, disk growth and block throughput at the target rate. Check encoded element sizes, sigops and modern computation budgets independently. Each candidate signature in the table exceeds the current 520-byte pre-Toccata element limit. [ARM benchmark and data](https://qanum.org/docs/benchmarks/2026-10-21_arm_second-verifier/) · [Script limits](../../crypto/txscript/src/lib.rs)

Android lists ML-DSA-65 and -87 starting at API 37 and requires eligible KeyMint 5+ hardware for the documented Android Keystore path. Apple lists both suites in Secure Enclave. Test availability, exact signing context and recovery policy on real devices. Compare hardware-backed signing with software signing protected by platform storage on older devices. Measure p50/p95 signing and verification, memory, binary growth, battery and thermal impact for 1, 2 and 10 input transactions on older ARM64, modern Android and iOS. Check QR receive and larger offline signing payloads independently. [Android API](https://developer.android.com/reference/android/security/keystore/KeyProperties) · [Android hardware requirements](https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec) · [Apple Secure Enclave](https://developer.apple.com/documentation/cryptokit/secureenclave)

## Post-launch research plan

1. Inventory transparent and shielded primitives and migration dependencies across wallet restore, indexing, custodians and offline signing.
2. Build an isolated default-off transparent Simnet prototype. Cross-check NIST vectors in two implementations and verify native, ARM64 and WASM agreement. Use a reviewed library path before considering production.
3. Publish reproducible byte, fee, legitimate and invalid verification, device, restoration and sustained multi-node results. Compare fixed-size output commitments and dual-signature policies.
4. Rehearse migration and mixed-version relay, then seek independent cryptographic review and separate owner approval for protocol activation. Evaluate shielded proof, encryption, and authorization migration separately. [NIST ACVP ML-DSA](https://pages.nist.gov/ACVP/draft-celi-acvp-ml-dsa.html) · [Open Quantum Safe](https://github.com/open-quantum-safe/liboqs) · [RustCrypto audit status](https://docs.rs/ml-dsa/latest/ml_dsa/)
