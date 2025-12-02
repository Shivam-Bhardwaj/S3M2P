# CRM - Customer Relationship Management

Customer relationship management for antimony-labs.

## Build & Run

```bash
trunk serve SW/CRM/index.html --open
trunk build --release SW/CRM/index.html
```

## Architecture

```
SW/CRM/
  src/
    lib.rs       # WASM entry point, core types
  index.html     # Entry point
```

## Core Types

### Contact
- id, name, email, phone, company
- notes, tags
- created_at, updated_at

### Deal
- id, title, contact_id
- value, stage, probability
- notes, timestamps

### DealStage
Lead -> Qualified -> Proposal -> Negotiation -> ClosedWon/ClosedLost

### Interaction
- Email, Call, Meeting, Note, Task
- Linked to contact

## Features (Planned)

- [ ] Contact management
- [ ] Deal pipeline
- [ ] Interaction history
- [ ] Task management
- [ ] Local storage persistence
- [ ] Export/Import
- [ ] Email integration
- [ ] Calendar sync

## Storage

Uses browser localStorage for persistence. Future: sync with STORAGE_SERVER.
