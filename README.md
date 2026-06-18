# client_kudos

## Project Title
client_kudos

## Project Description
Freelancers build their careers on reputation, but most of that reputation
lives in private, mutable, platform-locked databases that a client or
platform can rewrite at will. **client_kudos** is a Soroban smart contract
that turns client-given kudos into a public, append-only record on the
Stellar blockchain. After a project finishes, the client signs a single
transaction that publishes a 1-to-5 star rating, a project reference, and
a 32-byte hash of an off-chain comment. The freelancer then carries that
reputation with them across every platform, forever.

## Project Vision
We believe portable, verifiable professional reputation is foundational
infrastructure for the future of work. The long-term goal of
**client_kudos** is to become a neutral, open reputation layer that any
freelance marketplace, DAOs, or grant program can plug into — a public
good where trust is enforced by Stellar consensus instead of by any
single intermediary.

## Key Features
- **On-chain kudos issuance** — `give_kudos` lets a client sign and
  publish a star rating (1..=5), a `Symbol` project reference, and a
  32-byte hash of an off-chain comment in a single transaction.
- **Append-only history** — every kudos is stored in a per-freelancer
  `Vec<KudosRecord>`, so the full timeline is publicly auditable.
- **Client-controlled revocation** — `revoke_kudos` lets the *original*
  client soft-revoke a record with a reason; the history is preserved
  but the rating is removed from the running score.
- **Live reputation metrics** — `get_score`, `get_kudos_count`, and
  `get_average_rating` return the freelancer's current standing,
  with the average scaled by 100 to preserve two decimal places.
- **Authenticity verification** — `is_authentic` lets any third party
  confirm that a specific kudos was really issued by a claimed client.
- **No token transfers** — the contract never moves XLM or any custom
  asset; reputation is pure data, which keeps fees and complexity low.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** work dApp — see `contracts/client_kudos/src/lib.rs` for the full client_kudos business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** CBDK47KYNIJUIJ4TYZIT6HWJCB3FHYPSBBYCGWWCH6XMCLF5GAASHAFC
- **Explorer template:** https://stellar.expert/explorer/testnet/tx/8859877b75473a122899903e32e4c549959a1bfdd3bca41768bf377f8a8c0723
- **Screenshot of deployed contract on Stellar Expert:**
![screenshot](https://ibb.co/35Lzqwy8)


## Future Scope
- **Weighted ratings by client reputation** — weight each rating by the
  issuing client's own standing to resist new-account sybil attacks.
- **Optional USDC bounty** — let clients attach a small USDC tip to a
  kudos via a token transfer, paid out to the freelancer.
- **Skill tags** — extend `KudosRecord` with a `Vec<Symbol>` of skill
  tags so freelancers can build a per-skill reputation profile.
- **Off-chain comment anchoring** — store the IPFS / Arweave CID of the
  full comment alongside its hash for richer public feedback.
- **Frontend dApp** — build a React + Freighter UI for browsing a
  freelancer's history and submitting a new kudos without writing
  Soroban CLI commands.
- **Cross-chain bridge** — mirror kudos records to other networks so
  the reputation is readable by Ethereum-, Sui-, or Solana-based
  hiring platforms.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `client_kudos` (work)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
