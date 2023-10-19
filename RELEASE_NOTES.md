# 0.8.0-alpha.1

- Disable native staking in vault.
- Valset updates for external-staking.

# 0.7.0-alpha.2

- Remove empty messages / events.
- Fix virtual-staking slashing accounting.

# 0.7.0-alpha.1

- Cross-slashing implementation.
- Batch distribute rewards.
- Valset updates.
- Slashing accounting.
- Slashing propagation at the `vault` contract level.

# 0.3.0-beta

- IBC specification is added to the documents.
- IBC types and logic added to `mesh-api::ibc`
- `converter` and `external-staking` support IBC
  - Handshake and channel creation
  - Validator sync protocol (Consumer -> Provider)
    TODO: Dynamic updates
  - Staking protocol (Provider -> Consumer)
  - Rewards protocol (Consumer -> Provider -> Consumer)
