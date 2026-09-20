# Transactions — `Option<Transaction>` everywhere

Shifty manages transactions consistently at the service layer. Every
service method accepts `Option<Self::Transaction>`.

## The Canonical Pattern

```rust
async fn do_something(
    &self,
    dto: SomeDto,
    ctx: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;

    // ... business logic + DAO calls, tx.clone() each time ...

    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

### Behavior of `use_transaction`

- **`tx == None`:** A new TX is opened. The service is the owner.
  It commits itself at the end.
- **`tx == Some(existing)`:** The outer TX is passed through. The
  service is **not** the owner. The commit call at the end is then a
  no-op or a delegate that does not actually commit.

**Verified (v2.7, Phase 63).** The handle is an `Arc`, and
`TransactionDaoImpl::commit` only commits when it holds the **last**
reference:

```rust
async fn commit(&self, transaction: Self::Transaction) -> Result<(), DaoError> {
    if let Some(tx) = Arc::into_inner(transaction.tx) {
        tx.into_inner().commit().await.map_db_error()?;
    }
    Ok(())
}
```

An inner `commit` on a shared handle is therefore a genuine no-op — it only
drops one reference. The SQL `COMMIT` happens when the last holder lets go,
i.e. in the outermost service. `rollback` works the same way. **This is the
invariant that makes passing a transaction through possible at all**: a
business-logic service can hand `Some(tx.clone())` to a basic service whose
own code path ends in `commit`, without that basic service closing the
transaction behind its back.

## Why this way?

The advantage: **composition without re-opening**.

A business-logic service can call three basic services in sequence
without having to worry about whether each of them opens its own TX.
It opens **once** on the outside and passes it through:

```rust
// Business logic:
async fn create_booking_with_conflict_check(...) -> Result<..., ServiceError> {
    let tx = self.transaction_dao.use_transaction(None).await?;

    // All 3 calls run in the same TX
    self.absence_service.assert_no_conflict(sp, date, tx.clone()).await?;
    let booking = self.booking_service.create(dto, ctx.clone(), Some(tx.clone())).await?;
    self.booking_log_service.log_create(&booking, ctx.clone(), Some(tx.clone())).await?;

    self.transaction_dao.commit(tx).await?;
    Ok(booking)
}
```

If the absence check finds the conflict, the entire operation rolls
back — the booking was not created, nor was the log.

## Re-Point Atomicity

The same rule applies to **data moves**: slot splits, booking
migrations, anything that moves rows from one aggregate parent to
another.

**Rule:** Everything in ONE transaction. On mid-way failure: rollback,
all rows stay at the origin.

Without this rule you get intermediate states that double-count in
reports or make rows invisible. Phase 23 found this the hard way:
`modify_slot` dropped `max_paid_employees` because the update path
was not harmonized with the create path.

**[Convention]** For every re-point op there is an explicit test
against double-counting in reporting.

## Deliberate exception: `PermissionDao` is mixed

The convention above applies to **service** methods. On the DAO layer,
`dao::PermissionDao` is the one trait that is deliberately **mixed** (v2.7,
Phase 63):

| Method | Transaction |
| --- | --- |
| `create_user(user, person_id, process, tx)` | takes `Self::Transaction` |
| `delete_user(username, tx)` | takes `Self::Transaction` |
| everything else (`has_privilege`, `all_users`, `find_user`, role/privilege CRUD, `add_user_role`, …) | works on the pool directly |

**Why not all of them.** Only these two write paths are orchestrated by the
person administration, and only they have to sit in the same transaction as a
sales-person write or a person cleanup — the requirement is "after saving,
*exactly* the selected objects exist" plus the orphan cleanup of the `person`
row [D-63-01]. Converting the remaining fifteen methods would have touched
every caller in the workspace for no behavioural gain. The trait's doc comment
names the split, so a later reader does not mistake it for an unfinished
refactor.

**Why the associated type sits on the DAO and not on the service.**
`service::PermissionService` deliberately carries **no** `Transaction`
associated type. Roughly twenty services hold it as a dependency; adding one
there would force every single one of them to name a transaction type it does
not use. `PermissionServiceImpl` therefore holds a `TransactionDao` of its own
and opens/commits around the two DAO calls, so that its public surface stays
exactly as it was. See [`04-auth.md`](./04-auth.md).

## SQLite Specifics

SQLite serializes writes. Two parallel TXs that both want to write
lead to the second one receiving `BUSY`.

- **Consequence:** Long-running reports in a TX block writers. Only
  bundle read ops into a TX when consistency between multiple reads
  requires it (e.g. snapshot generation).
- **[To verify]** Retry behavior and timeouts in the `TransactionDao` impl.

## Errors & Rollback

- **`Result::Err`:** The ownership chain breaks, the TX is not
  committed. SQLite rolls back implicitly on Drop. **[To verify]**
  whether an explicit `Rollback` runs on Drop.
- **`panic!`:** Should not occur in services. If it does, the Drop
  semantics of the TX are critical. **[To verify]**

## Related edge cases

See [`../domain/edge-cases.md#7-transaktionen--atomarität`](../domain/edge-cases.md#7-transaktionen--atomarität).
