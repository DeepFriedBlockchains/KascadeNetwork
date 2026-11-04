# KASCADE Protocol & Fork Specification

Version: Draft 0.1 — unfinished, subject to change without notice

## 1. Identity

- Chain: KASCADE
- Coin: KCADE
- Smallest unit: KASMI
- 8 decimals; 1 KCADE = 100,000,000 KASMI
- Independent Layer-1 BlockDAG
- Target lineage: rusty-kaspa -\> ZKAS -\> KASCADE
- GHOSTDAG-derived consensus; kHeavyHash PoW
- Native mining always supported
- Target: 5 BPS / 200 ms
- Transparent + Orchard/Halo2-derived shielded transactions
- Kaspa-compatible AuxPoW target from genesis
- Transparent HRP: `kascade:`
- No premine, founder allocation, creator reward, ICO or private
  allocation
- Fair public mainnet launch

## 2. Mandatory upstream audit

Start from the current ZKAS source candidate and preserve required
license/provenance. Upstream lineage: `zkas-rusty` (ZKas), itself a fork of
`rusty-kaspa` (https://github.com/kaspanet/rusty-kaspa). See NOTICE.

Before consensus edits, create `UPSTREAM_AUDIT.md` with the exact
commit, clean-build/test results, network parameters,
genesis/address/port layout, BPS, maturity, DAA, GHOSTDAG,
finality/pruning, mass rules, Orchard state, AuxPoW, wallet/RPC,
emission code, and all consensus-sensitive files. Identify assumptions
caused by ZKAS being shielded-by-default. Do not guess unresolved
consensus values.

## 3. Monetary arithmetic

Consensus monetary arithmetic is integer KASMI only. Never use floating
point. - KASMI_PER_KCADE = 100,000,000 - MAX_SUPPLY_KCADE =
23,000,000,000 - MAX_SUPPLY_KASMI = 2,300,000,000,000,000,000 Audit
upstream integer types and overflow safety.

## 4. Cascading Emission

Primary issuance is exactly 21,000,000,000 KCADE. - Initial subsidy: 50
KCADE - Standard Cascade interval: 16,470,000 rewarding units, subject
to audit of the upstream reward clock - Reward sequence: 50, 49, 48 … 2,
1 KCADE - 16,470,000 \* sum(1..50) = 20,999,250,000 KCADE - Extend the
final 1-KCADE stage by 750,000 units - Total primary units:
824,250,000 - Exact primary issuance: 21B KCADE

Finite tail: - 0.5 KCADE per rewarding unit - Exactly 4,000,000,000 tail
units - Exact tail issuance: 2B KCADE - Subsidy becomes 0 permanently
afterward - Exact scheduled maximum: 23B KCADE

Add independent full-schedule tests and boundary vectors for every
Cascade, primary-to-tail, and tail-to-zero.

## 5. Reward distribution

Bootstrap: - Units 1-1500: 90% miner / 10% Development / 0% KRN - From
1501 onward, remain 90/10 until at least one verified active KRN exists

On the first eligible-KRN reward event, permanently enter Standard
Reward Mode: - 90% miner / 8% KRN / 2% Development

Never revert to Bootstrap Mode.

After Standard Reward Mode is active, if zero eligible KRNs exist: - 90%
miner / 8% KASCADE Community Fund (KCF) / 2% Development When KRNs
return, the 8% returns to KRN selection.

Integer rounding in Standard Mode: - development = floor(subsidy \* 2 /
100) - krn_or_kcf = floor(subsidy \* 8 / 100) - miner = subsidy -
development - krn_or_kcf Miner receives any indivisible remainder.

Bootstrap arithmetic must likewise sum exactly to subsidy.

Transaction fees initially go 100% to miners. Shielded and transparent
fees use the same allocation policy. No protocol fee burning.

## 6. Development Treasury

- 10% only during Bootstrap Mode; 2% during Standard Mode
- Ends with block subsidy
- New KASCADE transparent/unshielded address only; do not reuse any
  Kaspa address
- Target control: 3-of-5 multisig
- Public/explorer-visible address and spending history
- Final signers/address are TBD and must not be invented

## 7. KASCADE Community Fund (KCF)

Separate from Development Treasury. Receives the 8% KRN share only after
Standard Mode has activated and no eligible KRN exists. -
Transparent/unshielded KASCADE address - Target: 3-of-5 multisig -
Explorer-visible - Project-policy use: donations, airdrops, contests,
giveaways and publicly disclosed community/public-benefit initiatives -
Not team/founder compensation, salaries, creator rewards or ordinary
development funding - Final signers/address TBD Ordinary multisig does
not cryptographically enforce the allowed-purpose policy; documentation
must not claim that it does.

## 8. KASCADE Reward Nodes (KRN)

Capacity: - Maximum 300 simultaneously reward-eligible KRNs - One
qualifying KRN = one selection ticket - Extra collateral in one node
gives no extra reward weight

Collateral: - Exactly 30,000 KCADE per KRN - Locked/bonded on KASCADE;
operator retains ownership - Minimum commitment: 14 days - Nominal 5-BPS
equivalent: 6,048,000 units; audit correct consensus clock - Voluntary
exit after minimum commitment: 7-day unbond - Nominal 5-BPS equivalent:
3,024,000 units - No routine downtime slashing

Qualification: - Valid 30K collateral registration - Qualifying KASCADE
node service - 72 continuous hours of qualifying service before
eligibility - Nominal 5-BPS equivalent: 1,296,000 units -
Peer/connectivity/service checks - Registered operator key, reward
address, unique collateral - No extra 24-hour activation delay after the
72-hour qualification unless separately approved

Verification/downtime target: - Service verification about every 30
minutes (nominal 9,000 units) - 1-hour grace before reward suspension
(nominal 18,000) - 24 continuous hours offline =\> deactivated/removed
from reward eligibility - Downtime does NOT confiscate collateral -
Operator can restore and requalify for 72 hours, or exit/unbond subject
to commitment rules

Multiple KRNs per operator are allowed only with separate 30K collateral
and independently qualifying node/service identity per position. Design
anti-Sybil protections.

Do not consensus-lock hardware brands/models.

## 9. KRN deterministic lottery

One eligible KRN receives the entire 8% KRN share for each applicable
reward event.

User-facing model: every eligible KRN gets one lottery ticket per reward
event.

Implementation must be deterministic, reproducible, unbiased and
resistant to miner/operator grinding. Do not use literal “closest
without going over” if it introduces bias.

Before coding, create `KRN_SELECTION_DESIGN.md` covering: - eligible-set
snapshot/freeze rules - entropy from sufficiently old DAG state -
miner-grinding resistance - canonical KRN ordering - unbiased winner
mapping - BlockDAG/reorg behavior - duplicate/Sybil identity handling -
deterministic test vectors

Do not implement selection until this design is reviewed.

## 10. Transparent + shielded KCADE

Must support: - T -\> T - T -\> Z - Z -\> Z - Z -\> T

Retain/adapt secure ZKAS Orchard/Halo2 components. Shielded pool active
from genesis. Viewing keys and shielded memos supported.

Critical invariant: transparent value + shielded value + all
protocol-accounted value must reconcile to legitimate
issuance/transfers. No transition may create KCADE.

Audit domain separation, anchors, nullifiers, note commitments,
turnstile/supply accounting and BlockDAG reorg behavior. Benchmark
maximum shielded transaction complexity at 5 BPS.

## 11. Mining and AuxPoW

Native kHeavyHash mining always remains valid.

Target merged mining: - Kaspa-compatible AuxPoW from genesis - KASCADE
is an auxiliary chain but does not depend on Kaspa for liveness -
KASCADE difficulty remains independent - Goal: infrastructure may use
the same kHeavyHash work for Kaspa and auxiliary networks such as
ZKAS/KASCADE

Audit ZKAS AuxPoW files and pool/miner implementation before edits.
Create a unique KASCADE commitment/domain identifier.
Parent-confirmation assumptions are TBD and must not be invented.

## 12. 5-BPS conversion

Do not change only a BPS constant.

Audit/simulate at least: - GHOSTDAG K / mergeset limits - DAA and
minimum DAA window - past median time - timestamp deviation - finality
depth - merge depth - pruning depth - coinbase maturity - block
mass/lanes - shielded maturity - wallet/RPC confirmations - pool/miner
timing - P2P propagation

Use devnet/simnet/override/simulation facilities where available.
Produce `KASCADE_5BPS_REPORT.md` with proposed values and evidence.
GHOSTDAG/DAA/finality/pruning/timestamp values are intentionally not
preselected.

## 13. Maturity

Final coinbase and shielded maturity values are TBD pending the 5-BPS
audit. KRN reward maturity should initially match mining reward maturity
unless a documented security reason requires otherwise. KRN collateral
registration must use actually spendable funds, not merely nominally
emitted subsidy.

## 14. Network isolation

Required unique KASCADE values: - network ID and magic - genesis - P2P
and RPC/gRPC ports - test/dev/sim ports - data directories - mainnet HRP
`kascade:` - separate test/dev/sim HRPs - shielded address/domain
encoding - private-key/WIF behavior as applicable - HD derivation/coin
type - DNS/bootstrap seeds

Exact values other than `kascade:` are TBD pending audit. Never reuse
the previously discussed `kaspa:` treasury address.

## 15. Genesis

- No premine
- No founder/developer genesis allocation
- Generate final mainnet genesis only after consensus freeze
- Use actual mainnet genesis date Preferred message tone: “Born from
  Kaspa. Built as KASCADE. The BlockDAG gains a new sibling.” Exact
  message/date/hash remain TBD until genesis.

## 16. Infrastructure

Ports, network magic, DNS seeds and bootstrap peers are TBD. Operational
goal: multiple independent bootstrap nodes; 3-5
geographically/provider-diverse nodes is a reasonable starting target,
not a consensus requirement.

## 17. Fees, dust and mass

AUDIT/TBD: - minimum relay fee - dust rules - maximum block mass/size -
maximum transaction mass - shielded complexity limits Prefer
upstream-safe behavior initially, benchmarked for 5 BPS. Keep
relay/mempool policy separate from consensus where possible.

## 18. Burning

- No automatic protocol/fee burn
- Voluntary provably unspendable burns may be supported
- Any displayed KASCADE burn address must be provably unspendable, not
  merely a supposedly lost key
- Burns do not permit replacement issuance above the emission schedule

## 19. Governance/upgrades

- No on-chain governance initially
- No consensus-level emergency developer key
- Versioned hard-fork activation
- Exact activation mechanism TBD; prefer deterministic DAA/state-based
  activation consistent with upstream architecture

## 20. Launch

- Fair public mainnet mining launch
- Experienced Kaspa/ZKAS miners may participate in private/public
  testnets
- No exclusive valuable mainnet mining before public genesis

## 21. Required testnet coverage

Test: - multi-node sync - native mining - AuxPoW - sustained 5 BPS -
partitions/rejoins - reorg/DAG ordering - T-\>T, T-\>Z, Z-\>Z, Z-\>T -
malformed privacy proofs - supply invariants - every emission boundary -
KRN lifecycle - 300-KRN cap - KRN lottery determinism -
suspension/requalification/unbonding - KCF fallback - reward splits -
wallet/RPC/explorer - testnet faucet

## 22. Explorer

Expose: - circulating issuance and hard-cap progress - current
subsidy/Cascade - miner/KRN/Development/KCF outputs - active KRN count
and collateral locked - labeled Development Treasury and KCF
balances/history - transparent transaction data - safe non-private
shielded metadata - native/AuxPoW information where useful - network
BPS/consensus stats

## 23. Wallet/RPC

Eventually support: - transparent/shielded/spendable balances - locked
KRN collateral - send/receive/shield/unshield - viewing keys - KRN
registration/status/exit/unbonding - maturity/confirmation states
Version/document new APIs.

## 24. Implementation phases

0.  Audit/reproduce upstream
1.  KASCADE naming/network isolation/devnet
2.  5-BPS engineering/simulation
3.  Cascading Emission/hard-cap
4.  Bootstrap + reward splits
5.  Transparent + shielded model
6.  KASCADE AuxPoW/domain separation
7.  KRN registry/collateral/qualification
8.  KRN lottery + KCF fallback
9.  Wallet/RPC/explorer/pool tooling
10. Adversarial public testnet/security review
11. Mainnet genesis only after explicit approval

Each phase must build, test, document consensus impact and be committed
separately.

## 25. Non-negotiable invariants

1.  Independent KASCADE chain
2.  No premine
3.  No founder/creator allocation
4.  Exact scheduled maximum 23B KCADE
5.  Integer KASMI monetary consensus
6.  Native kHeavyHash remains valid
7.  5-BPS target with engineered supporting parameters
8.  Transparent/shielded coexistence cannot create supply
9.  30K KCADE KRN collateral
10. Maximum 300 eligible KRNs
11. Routine downtime never confiscates collateral
12. Standard Reward Mode never reverts to Bootstrap Mode
13. No-KRN fallback after activation goes to transparent KCF
14. No consensus developer emergency key
15. No final mainnet genesis before consensus freeze

## 26. Explicit approval required before mainnet

GHOSTDAG/mergeset parameters; DAA; finality/merge/pruning; timestamp
rules; maturity; mass limits; fee/dust policy; exact KRN verification;
lottery entropy/snapshot design; AuxPoW commitment; parent assumptions;
network magic/IDs; ports; shielded domains/addresses; WIF; HD path;
treasury/KCF multisig addresses/signers; seeds; genesis; testnet
results; security-review findings.

## 27. Security gate

A successful compile is not a security audit. Before mainnet obtain
independent review of Orchard/Halo2 adaptation, transparent/shielded
conservation, BlockDAG reorg behavior, AuxPoW/domain separation, 5-BPS
consensus behavior, KRN state machine/selection and exact hard-cap
arithmetic.
