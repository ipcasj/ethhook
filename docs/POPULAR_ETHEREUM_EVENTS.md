# Popular Ethereum Events for Webhook Platform

## Currently Implemented

- ✅ **Transfer** - ERC-20/ERC-721 token transfers

## Top 10 Most Popular Events to Add

### 1. **Approval** (ERC-20/ERC-721)

**Event Signature**: `Approval(address indexed owner, address indexed spender, uint256 value)`
**Keccak-256 Hash**: `0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925`

**Use Cases**:

- Track when users approve tokens for DeFi protocols (Uniswap, Aave, etc.)
- Monitor security: Detect unlimited approvals
- Notify users of approval changes

**Example**: User approves Uniswap Router to spend their USDC

---

### 2. **Swap** (Uniswap V2/V3)

**Event Signature**: `Swap(address indexed sender, uint amount0In, uint amount1In, uint amount0Out, uint amount1Out, address indexed to)`
**Keccak-256 Hash**: `0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822` (Uniswap V2)

**Use Cases**:

- Monitor DEX trading activity
- Track DEX trades in real-time
- Price feed monitoring
- Arbitrage opportunity detection
- Trading volume analytics

**Example**: User swaps 1000 USDC for 0.3 ETH on Uniswap

---

### 3. **Deposit** (WETH, Staking Contracts)

**Event Signature**: `Deposit(address indexed dst, uint256 wad)`
**Keccak-256 Hash**: `0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c`

**Use Cases**:

- Monitor ETH wrapping to WETH
- Track staking deposits
- Liquidity pool deposits

**Example**: User deposits 10 ETH to wrap as WETH

---

### 4. **Withdrawal** (WETH, Staking Contracts)

**Event Signature**: `Withdrawal(address indexed src, uint256 wad)`
**Keccak-256 Hash**: `0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65`

**Use Cases**:

- Monitor WETH unwrapping
- Track unstaking/withdrawals
- Liquidity removal

**Example**: User withdraws 5 WETH and converts to ETH

---

### 5. **ApprovalForAll** (ERC-721/ERC-1155 NFTs)

**Event Signature**: `ApprovalForAll(address indexed owner, address indexed operator, bool approved)`
**Keccak-256 Hash**: `0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31`

**Use Cases**:

- NFT marketplace approvals
- Security monitoring (detect malicious approvals)
- Track OpenSea/Blur marketplace interactions

**Example**: User approves OpenSea to trade all their NFTs

---

### 6. **OwnershipTransferred** (Access Control)

**Event Signature**: `OwnershipTransferred(address indexed previousOwner, address indexed newOwner)`
**Keccak-256 Hash**: `0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0`

**Use Cases**:

- Smart contract governance monitoring
- Security alerts (ownership changes)
- Project updates tracking

**Example**: DAO multisig takes ownership of protocol contract

---

### 7. **RoleGranted** / **RoleRevoked** (Access Control)

**RoleGranted Signature**: `RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)`
**Keccak-256 Hash**: `0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d`

**Use Cases**:

- Admin role changes
- Governance actions
- Security monitoring

**Example**: New admin added to DeFi protocol

---

### 8. **Paused** / **Unpaused** (Circuit Breaker)

**Paused Signature**: `Paused(address account)`
**Keccak-256 Hash**: `0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258`

**Use Cases**:

- Emergency monitoring
- Protocol status tracking
- Downtime alerts

**Example**: Protocol paused due to security incident

---

### 9. **Mint** / **Burn** (Token Supply Changes)

**Mint Signature**: `Mint(address indexed to, uint256 amount)` (varies by implementation)
**Burn Signature**: `Burn(address indexed from, uint256 amount)`

**Use Cases**:

- Track token creation events
- Track token minting events
- Monitor token supply changes
- Tokenomics tracking
- Token supply monitoring
- Stablecoin minting/burning
- Deflationary token mechanics

**Example**: Circle mints 100M USDC

---

### 10. **ProposalCreated** / **VoteCast** (Governance)

**ProposalCreated Signature**: `ProposalCreated(uint256 proposalId, address proposer, ...)`
**VoteCast Signature**: `VoteCast(address indexed voter, uint256 proposalId, uint8 support, uint256 weight, string reason)`

**Use Cases**:

- DAO governance tracking
- Voting power analysis
- Proposal monitoring

**Example**: New governance proposal for Compound Protocol

---

## Events by Category

### DeFi Events
1. **Swap** - DEX trades
2. **Deposit/Withdrawal** - Liquidity management
3. **Borrow/Repay** (Aave, Compound) - Lending protocols
4. **Liquidation** - Undercollateralized positions

### NFT Events
1. **Transfer** (ERC-721) - NFT ownership changes
2. **ApprovalForAll** - Marketplace approvals
3. **Mint** - New NFT creation
4. **Burn** - NFT destruction

### Token Events
1. **Transfer** - Token movements
2. **Approval** - Spending permissions
3. **Mint/Burn** - Supply changes

### Governance Events
1. **ProposalCreated** - New proposals
2. **VoteCast** - Voting activity
3. **RoleGranted/Revoked** - Permission changes

### Security Events
1. **Paused/Unpaused** - Emergency stops
2. **OwnershipTransferred** - Control changes
3. **ApprovalForAll** - Risky permissions

---

## Recommended Implementation Priority

### Phase 1 (High Impact, Easy)
1. ✅ **Transfer** (already done)
2. **Approval** - Very common, easy to implement
3. **Swap** - High value for DeFi users
4. **Deposit/Withdrawal** - Common patterns

### Phase 2 (Popular Use Cases)
5. **ApprovalForAll** - NFT marketplace activity
6. **OwnershipTransferred** - Security monitoring
7. **Paused/Unpaused** - Critical alerts

### Phase 3 (Advanced)
8. **RoleGranted/Revoked** - Governance
9. **ProposalCreated/VoteCast** - DAO activity
10. **Liquidation** - DeFi risk management

---

## Sepolia Testnet Contract Suggestions

### For Demo/MVP, Add These Endpoints:

#### 1. WETH Approval Events
- Contract: `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9` (Sepolia WETH)
- Event: `Approval`
- Use Case: Monitor when users approve spending

#### 2. WETH Deposit/Withdrawal Events
- Contract: `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9` (Sepolia WETH)
- Events: `Deposit`, `Withdrawal`
- Use Case: Track ETH wrapping/unwrapping

#### 3. Uniswap V2 Pair Swap Events (if available on Sepolia)
- Find active DEX pairs on Sepolia
- Event: `Swap`
- Use Case: Track test trading activity

---

## SQL Script to Add Popular Events

```sql
-- Add WETH Approval tracking
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    hmac_secret,
    chain_ids,
    contract_addresses,
    event_signatures,
    is_active
) VALUES (
    (SELECT id FROM applications WHERE email = 'demo@ethhook.com'),
    'Sepolia WETH Approvals',
    'https://webhook.site/YOUR-UNIQUE-URL',
    encode(gen_random_bytes(32), 'hex'),
    ARRAY[11155111],
    ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'],
    ARRAY['0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925'],
    true
);

-- Add WETH Deposit tracking
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    hmac_secret,
    chain_ids,
    contract_addresses,
    event_signatures,
    is_active
) VALUES (
    (SELECT id FROM applications WHERE email = 'demo@ethhook.com'),
    'Sepolia WETH Deposits',
    'https://webhook.site/YOUR-UNIQUE-URL',
    encode(gen_random_bytes(32), 'hex'),
    ARRAY[11155111],
    ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'],
    ARRAY['0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c'],
    true
);

-- Add WETH Withdrawal tracking
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    hmac_secret,
    chain_ids,
    contract_addresses,
    event_signatures,
    is_active
) VALUES (
    (SELECT id FROM applications WHERE email = 'demo@ethhook.com'),
    'Sepolia WETH Withdrawals',
    'https://webhook.site/YOUR-UNIQUE-URL',
    encode(gen_random_bytes(32), 'hex'),
    ARRAY[11155111],
    ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'],
    ARRAY['0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65'],
    true
);

-- Add USDC Approval tracking
INSERT INTO endpoints (
    application_id,
    name,
    webhook_url,
    hmac_secret,
    chain_ids,
    contract_addresses,
    event_signatures,
    is_active
) VALUES (
    (SELECT id FROM applications WHERE email = 'demo@ethhook.com'),
    'Sepolia USDC Approvals',
    'https://webhook.site/YOUR-UNIQUE-URL',
    encode(gen_random_bytes(32), 'hex'),
    ARRAY[11155111],
    ARRAY['0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238'],
    ARRAY['0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925'],
    true
);
```

---

## Event Signature Reference (Quick Copy)

```
Transfer:           0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
Approval:           0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
Swap (Uni V2):      0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822
Deposit (WETH):     0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c
Withdrawal (WETH):  0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65
ApprovalForAll:     0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31
OwnershipTransfer:  0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0
Paused:             0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258
RoleGranted:        0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d
```

---

## Benefits of Multiple Event Types

1. **Richer Demo**: Shows platform versatility beyond just transfers
2. **Real Use Cases**: Matches actual customer needs (DeFi, NFT, Security monitoring)
3. **Better Testing**: More event types = more webhook deliveries to demo
4. **Competitive Edge**: Most webhook platforms focus on transfers; you support all events
5. **Upsell Opportunities**: Advanced event types can be premium features

---

## Next Steps for MVP

1. Add 3-5 event types (Approval, Deposit, Withdrawal as minimum)
2. Update dashboard to show event type breakdown
3. Add event type filter in Events page UI
4. Update documentation with event type examples
5. Create demo webhooks for each event type
