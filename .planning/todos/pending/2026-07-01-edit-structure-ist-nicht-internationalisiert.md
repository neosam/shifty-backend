---
created: 2026-07-01T04:29:55.804Z
title: "Edit structure" ist noch nicht internationalisiert
area: frontend
resolves_phase: 46
files:
  - shifty-dioxus/
---

## Problem

Die "Edit structure"-Funktion im Dioxus-Frontend ist noch nicht internationalisiert.
Texte (Labels, Buttons, Überschriften) sind hartkodiert und fehlen in der i18n-Struktur,
sodass sie nicht in allen drei Locales (En, De, Cs) übersetzt vorliegen.

Gemäß Projekt-Konvention müssen alle Texte über i18n laufen und Übersetzungen für
En/De/Cs vorhanden sein.

## Solution

TBD — Hartkodierte Strings der "Edit structure"-UI identifizieren, in die i18n-Keys
extrahieren und Übersetzungen für alle drei Locales (En, De, Cs) ergänzen.
