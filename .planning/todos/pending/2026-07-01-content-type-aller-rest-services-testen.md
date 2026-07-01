---
created: 2026-07-01T04:40:00.000Z
title: Content-Type aller REST-Services testen und korrigieren
area: api
files:
  - rest/
---

## Problem

Die REST-Endpunkte setzen nicht alle den korrekten `Content-Type` in ihrer Response.
Einige Handler liefern vermutlich einen falschen oder fehlenden Content-Type
(z. B. `text/plain` statt `application/json`, oder umgekehrt bei nicht-JSON-Payloads
wie PDF/CSV/Text).

Das kann Clients (insbesondere das Dioxus-Frontend) beim Parsen der Response stören
und ist inkonsistent über die Endpunkte hinweg.

## Solution

TBD — Alle REST-Handler in `rest/` durchgehen und den gesetzten Content-Type prüfen.
Tests ergänzen, die pro Endpunkt den `Content-Type`-Header der Response asserten
(JSON-Endpunkte → `application/json`, Datei-/Text-Endpunkte → passender MIME-Typ).
Falsche/fehlende Content-Types korrigieren.
