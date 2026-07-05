# Transaktionen — `Option<Transaction>` überall

Shifty verwaltet Transaktionen konsequent auf der Service-Ebene. Jede
Service-Methode akzeptiert `Option<Self::Transaction>`.

## Das Kanonische Pattern

```rust
async fn do_something(
    &self,
    dto: SomeDto,
    ctx: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;

    // ... business logic + DAO calls, jedes Mal tx.clone() ...

    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

### Verhalten von `use_transaction`

- **`tx == None`:** Es wird eine neue TX geöffnet. Der Service ist
  Owner. Am Ende committet er selbst.
- **`tx == Some(existing)`:** Die äußere TX wird durchgereicht. Der
  Service ist **nicht** Owner. Der Commit-Aufruf am Ende ist dann ein
  No-op oder ein Delegate, das nicht wirklich committet.

**[Zu prüfen]** Die exakte Semantik in der `TransactionDao`-Impl —
konkret ob `commit` bei durchgereichtem TX kein tatsächliches SQL
`COMMIT` fährt. Das ist die kritische Invariante, ohne die das Pattern
kaputt ist.

## Warum so?

Der Vorteil: **Komposition ohne Neu-Öffnung**.

Ein Business-Logic-Service kann drei Basic-Services in Folge aufrufen
und muss sich nicht darum kümmern, ob jeder von denen eine eigene TX
öffnet. Er öffnet **einmal** außen und reicht durch:

```rust
// Business-Logic:
async fn create_booking_with_conflict_check(...) -> Result<..., ServiceError> {
    let tx = self.transaction_dao.use_transaction(None).await?;

    // Alle 3 Aufrufe fahren in derselben TX
    self.absence_service.assert_no_conflict(sp, date, tx.clone()).await?;
    let booking = self.booking_service.create(dto, ctx.clone(), Some(tx.clone())).await?;
    self.booking_log_service.log_create(&booking, ctx.clone(), Some(tx.clone())).await?;

    self.transaction_dao.commit(tx).await?;
    Ok(booking)
}
```

Wenn der Absence-Check den Konflikt findet, rollt die gesamte
Operation zurück — Booking wurde nicht angelegt, Log auch nicht.

## Re-Point-Atomarität

Dieselbe Regel gilt für **Datenumzüge**: Slot-Split, Booking-Migration,
alles, was Zeilen von einem Aggregat-Parent zu einem anderen bewegt.

**Regel:** Alles in EINER Transaktion. Bei Fehler mid-way: Rollback,
alle Zeilen bleiben am Ursprung.

Ohne diese Regel gibt es Zwischenzustände, die Reports doppelt zählen
oder Zeilen unsichtbar machen. Phase 23 hat das schmerzlich gefunden:
`modify_slot` ließ `max_paid_employees` fallen, weil der Update-Pfad
nicht mit dem Create-Pfad harmonisiert war.

**Konvention:** Für jede Re-Point-Op gibt es einen expliziten Test
gegen Doppelzählung im Reporting.

## SQLite-Spezifika

SQLite serialisiert Writes. Zwei parallele TX, die beide schreiben
wollen, führen dazu, dass die zweite `BUSY` erhält.

- **Konsequenz:** Long-running-Reports in einer TX blockieren Writer.
  Read-Ops nur dann in eine TX bündeln, wenn Konsistenz zwischen
  mehreren Reads das erfordert (z.B. Snapshot-Erzeugung).
- **[Zu prüfen]** Retry-Verhalten und Timeouts in der `TransactionDao`-Impl.

## Fehler & Rollback

- **`Result::Err`:** Ownership-Kette bricht ab, TX wird nicht committet.
  SQLite rollt bei Drop implizit zurück. **[Zu prüfen]** ob explizites
  `Rollback` auf Drop läuft.
- **`panic!`:** Sollte in Services nicht vorkommen. Falls doch, ist die
  Drop-Semantik der TX kritisch. **[Zu prüfen]**

## Verwandte Randfälle

Siehe [`../domain/edge-cases.md#7-transaktionen--atomarität`](../domain/edge-cases.md#7-transaktionen--atomarität).
