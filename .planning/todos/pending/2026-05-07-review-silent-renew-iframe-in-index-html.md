---
created: 2026-05-07T10:50:00
title: "Review: silentRenewIframe in index.html — Zweck und Verwendung unklar"
area: frontend / auth
files:
  - shifty-dioxus/index.html:29
  - shifty-dioxus/index.html:31-39
  - shifty-dioxus/flake.nix:117-134
---

## Beobachtung

Aus Frontend-Codebase-Map, nicht als Bug verifiziert.
**Bitte einschätzen: löschen, dokumentieren oder anders behandeln?**

`shifty-dioxus/index.html:29` enthält:

```html
<iframe id="silentRenewIframe" style="display:none;"></iframe>
<script>
    window.oidcLoginKeepAliveURL = null;
    setInterval(() => {
        if (window.oidcLoginKeepAliveURL) {
            fetch(oidcLoginKeepAliveURL, { method: "GET" });
        }
    }, 1000 * 60 * 5);
</script>
```

Status nach Code-Inspektion:

- Der iframe wird nirgendwo befüllt — kein `src`-Attribut, keine Stelle
  im Rust-Code, die `web-sys` o. ä. nutzt um seine URL zu setzen.
- `window.oidcLoginKeepAliveURL` wird im JS auf `null` initialisiert und
  vom Rust-Code nicht überschrieben (grep nach `oidcLoginKeepAlive` im
  `src/` ergibt keine Treffer).
- Der `flake.nix`-Production-Build überschreibt `dist/index.html` mit
  einem minimalen Template (`flake.nix:117-134`), das den iframe und das
  Keep-Alive-Script **nicht** enthält.

Heißt: Beides ist heute „dead-on-arrival" Vorbereitungs-Code, vermutlich
aus einem früheren OIDC-Browser-side-Setup (oidc-client-ts o. ä.). Das
aktuelle Auth-Modell ist Backend-Session-Cookie + `/auth-info`-Ping +
401-getriggerter Reload (siehe
`.planning/codebase/frontend/INTEGRATIONS.md`). Renewal passiert
serverseitig, nicht via iframe.

**Bestätigt: produktiv stabil, kein User-sichtbarer Bug.**

## Frage an dich

1. War das mal ein anderer Auth-Flow, der nicht mehr aktiv ist? Falls ja:
   beide Code-Snippets (iframe + Keep-Alive-Script) entfernen, weil tot.
2. Oder ist es **bewusst** als Hook für zukünftige Browser-side-Renewal
   stehengeblieben? Dann gehört eine Code-Notiz dran, dass die Variable
   extern gesetzt werden muss, sonst „why is this here" beim nächsten Lesen.
3. Diskrepanz zwischen Source-`index.html` und Nix-Built-`index.html`:
   wird der Nix-Pfad produktiv genutzt, oder läuft der Production-Build
   eigentlich über `dx build` mit dem echten Source-Template?
   - Wenn Nix nicht produktiv → Nix-Build vereinfachen oder löschen
   - Wenn Nix produktiv → Source-`index.html` ist redundant

## Nicht in Scope

- Größerer OIDC-Refactor — heute läuft alles produktiv stabil.
