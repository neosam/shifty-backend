//! Frontend-Form-State und pure Konvertierungs-Funktionen für die
//! Nextcloud-PDF-Export-Config-Card (Phase 48-05, EXP-02/EXP-03).
//!
//! Der Backend-DTO `PdfExportConfigTO` (aus `rest-types`) ist die Serialisierungs-
//! Form. Diese Datei stellt die **UI-Form-Repräsentation** bereit, in der:
//!
//! - alle Text-Felder als `String` (statt `Option<Arc<str>>`) mit Dioxus-Signalen
//!   bindbar sind,
//! - `token_input` ein reines UI-Feld ist (Klartext, wird bei leerem Feld als
//!   `None` in den PUT-Body übersetzt = „Token behalten" per D-48-UI-TOKEN-KEEP),
//! - die read-only Status-Felder (`last_success_at`, `last_error_at`,
//!   `last_error_message`) 1:1 aus der GET-Response übernommen werden.
//!
//! Pure Funktionen `pdf_export_form_to_put_body`, `pdf_export_form_from_response`
//! und `clamp_weeks_horizon` sind Unit-getestet (Presence-Test der neuen i18n-Keys
//! liegt separat in `i18n/mod.rs`).

use std::sync::Arc;

use rest_types::PdfExportConfigTO;
use time::PrimitiveDateTime;

/// UI-Form-Repräsentation der PDF-Export-Config.
///
/// Alle Text-Felder sind `String`, damit Dioxus-Signale sie direkt binden können.
/// Der `token_input` ist immer initial leer (Response maskiert den Token IMMER
/// per T-48-02) — der Placeholder in der UI erklärt „(unverändert, hier neues
/// Token eintippen)".
#[derive(Clone, Debug, PartialEq)]
pub struct PdfExportForm {
    pub enabled: bool,
    pub nextcloud_url: String,
    pub webdav_user: String,
    /// Klartext-Eingabe im Passwort-Feld. Leer = „Token unverändert lassen"
    /// (siehe `pdf_export_form_to_put_body`). Nach jedem Save wieder leer,
    /// weil die GET-Response den Token IMMER als `None` liefert.
    pub token_input: String,
    pub target_folder: String,
    pub weeks_horizon: u32,
    pub cron_schedule: String,
    // Read-only Status (aus Server-Response)
    pub last_success_at: Option<PrimitiveDateTime>,
    pub last_error_at: Option<PrimitiveDateTime>,
    pub last_error_message: Option<String>,
}

impl Default for PdfExportForm {
    fn default() -> Self {
        Self {
            enabled: false,
            nextcloud_url: String::new(),
            webdav_user: String::new(),
            token_input: String::new(),
            target_folder: String::new(),
            weeks_horizon: 8,
            cron_schedule: "0 6 * * 1".to_string(),
            last_success_at: None,
            last_error_at: None,
            last_error_message: None,
        }
    }
}

/// Übersetzt die UI-Form in einen PUT-Request-Body (`PdfExportConfigTO`).
///
/// Token-Merge-Semantik (D-48-UI-TOKEN-KEEP):
/// - `token_input` leer → `webdav_app_token = None` → Backend behält den
///   bestehenden Token (siehe Plan 48-01 D4).
/// - `token_input` nicht leer → `webdav_app_token = Some(new)` → Backend
///   überschreibt.
///
/// Die read-only Status-Felder werden 1:1 mitgesendet (der Backend-Service
/// ignoriert sie im PUT-Handler; sie werden nur von `record_success` /
/// `record_error` geschrieben — kein Read-Leak, kein Konflikt).
pub fn pdf_export_form_to_put_body(form: &PdfExportForm) -> PdfExportConfigTO {
    fn none_if_empty(s: &str) -> Option<Arc<str>> {
        if s.is_empty() {
            None
        } else {
            Some(Arc::from(s))
        }
    }
    PdfExportConfigTO {
        enabled: form.enabled,
        nextcloud_url: none_if_empty(&form.nextcloud_url),
        webdav_user: none_if_empty(&form.webdav_user),
        webdav_app_token: if form.token_input.is_empty() {
            None
        } else {
            Some(Arc::from(form.token_input.as_str()))
        },
        target_folder: none_if_empty(&form.target_folder),
        weeks_horizon: clamp_weeks_horizon(form.weeks_horizon as i32),
        cron_schedule: Arc::from(form.cron_schedule.as_str()),
        last_success_at: form.last_success_at,
        last_error_at: form.last_error_at,
        last_error_message: form.last_error_message.as_deref().map(Arc::from),
    }
}

/// Übersetzt die Server-Response in die UI-Form.
///
/// - `webdav_app_token` ist im Response IMMER `None` (T-48-02, Plan 48-01
///   `From<&PdfExportConfig>`); `token_input` bleibt leer, sodass ein direkter
///   nachfolgender Save den bestehenden Token behält.
/// - `nextcloud_url` / `webdav_user` / `target_folder` werden aus `Option<Arc<str>>`
///   in `String` gefaltet (leer wenn `None`).
pub fn pdf_export_form_from_response(response: &PdfExportConfigTO) -> PdfExportForm {
    fn arc_to_string(v: &Option<Arc<str>>) -> String {
        v.as_ref().map(|s| s.as_ref().to_string()).unwrap_or_default()
    }
    PdfExportForm {
        enabled: response.enabled,
        nextcloud_url: arc_to_string(&response.nextcloud_url),
        webdav_user: arc_to_string(&response.webdav_user),
        token_input: String::new(),
        target_folder: arc_to_string(&response.target_folder),
        weeks_horizon: response.weeks_horizon,
        cron_schedule: response.cron_schedule.as_ref().to_string(),
        last_success_at: response.last_success_at,
        last_error_at: response.last_error_at,
        last_error_message: response
            .last_error_message
            .as_ref()
            .map(|s| s.as_ref().to_string()),
    }
}

/// Clamp Wochen-Horizont auf 1..=52 (D-48-UI-FIELDS: 1..=52 Range).
///
/// Für negative Eingaben oder `< 1` → 1, für `> 52` → 52. Damit ist der
/// PUT-Body immer im gültigen Range, unabhängig davon, was der Browser
/// (min/max) durchlässt.
pub fn clamp_weeks_horizon(input: i32) -> u32 {
    if input < 1 {
        1
    } else if input > 52 {
        52
    } else {
        input as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Month, Time};

    fn some_datetime() -> PrimitiveDateTime {
        PrimitiveDateTime::new(
            Date::from_calendar_date(2026, Month::July, 3).unwrap(),
            Time::from_hms(10, 0, 0).unwrap(),
        )
    }

    // ── Test 1 (pure): config_form_to_put_body_omits_empty_token ─────────

    /// D-48-UI-TOKEN-KEEP: leerer token_input → webdav_app_token = None
    /// (Backend behält den bestehenden Wert).
    #[test]
    fn pdf_export_form_to_put_body_empty_token_becomes_none() {
        let form = PdfExportForm {
            enabled: true,
            nextcloud_url: "https://example.org/dav".to_string(),
            webdav_user: "user".to_string(),
            token_input: String::new(),
            target_folder: "Schichtplaene".to_string(),
            weeks_horizon: 8,
            cron_schedule: "0 6 * * 1".to_string(),
            ..PdfExportForm::default()
        };
        let body = pdf_export_form_to_put_body(&form);
        assert!(body.webdav_app_token.is_none());
        assert!(body.enabled);
        assert_eq!(
            body.nextcloud_url.as_deref().map(|s| s.to_string()),
            Some("https://example.org/dav".to_string())
        );
    }

    /// D-48-UI-TOKEN-KEEP: nicht-leerer token_input → Some(new).
    #[test]
    fn pdf_export_form_to_put_body_nonempty_token_becomes_some() {
        let mut form = PdfExportForm::default();
        form.token_input = "secret".to_string();
        let body = pdf_export_form_to_put_body(&form);
        assert_eq!(
            body.webdav_app_token.as_deref().map(|s| s.to_string()),
            Some("secret".to_string())
        );
    }

    // ── Test 2 (pure): config_form_from_response_leaves_token_input_empty ─

    /// T-48-02: Server maskiert Token IMMER; die UI-Form startet mit leerem
    /// `token_input`, damit direkte Saves den Token nicht überschreiben.
    #[test]
    fn pdf_export_form_from_response_leaves_token_input_empty() {
        let response = PdfExportConfigTO {
            enabled: true,
            nextcloud_url: Some(Arc::from("https://x/dav")),
            webdav_user: Some(Arc::from("u")),
            webdav_app_token: None, // Server-response ist IMMER None
            target_folder: Some(Arc::from("folder")),
            weeks_horizon: 12,
            cron_schedule: Arc::from("0 5 * * 1"),
            last_success_at: Some(some_datetime()),
            last_error_at: None,
            last_error_message: None,
        };
        let form = pdf_export_form_from_response(&response);
        assert_eq!(form.token_input, "");
        assert_eq!(form.nextcloud_url, "https://x/dav");
        assert_eq!(form.webdav_user, "u");
        assert_eq!(form.target_folder, "folder");
        assert_eq!(form.weeks_horizon, 12);
        assert_eq!(form.cron_schedule, "0 5 * * 1");
        assert!(form.enabled);
        assert_eq!(form.last_success_at, Some(some_datetime()));
    }

    /// Server-Response ohne Werte → Form ist "leer" mit den unveränderten
    /// weeks_horizon/cron aus der Response (die Backend-Migration seedet mit
    /// 8 / "0 6 * * 1"). Wenn die Response weeks=0 liefert, spiegelt die Form
    /// das 1:1 wider — clamp geschieht erst beim `_to_put_body`.
    #[test]
    fn pdf_export_form_from_response_none_fields_become_empty_strings() {
        let response = PdfExportConfigTO {
            enabled: false,
            nextcloud_url: None,
            webdav_user: None,
            webdav_app_token: None,
            target_folder: None,
            weeks_horizon: 8,
            cron_schedule: Arc::from("0 6 * * 1"),
            last_success_at: None,
            last_error_at: None,
            last_error_message: None,
        };
        let form = pdf_export_form_from_response(&response);
        assert_eq!(form.nextcloud_url, "");
        assert_eq!(form.webdav_user, "");
        assert_eq!(form.target_folder, "");
        assert_eq!(form.token_input, "");
        assert!(!form.enabled);
    }

    // ── Test 3 (pure): weeks_horizon_clamp ───────────────────────────────

    /// D-48-UI-FIELDS: 1..=52 Range.
    #[test]
    fn clamp_weeks_horizon_boundaries() {
        assert_eq!(clamp_weeks_horizon(0), 1);
        assert_eq!(clamp_weeks_horizon(-5), 1);
        assert_eq!(clamp_weeks_horizon(1), 1);
        assert_eq!(clamp_weeks_horizon(8), 8);
        assert_eq!(clamp_weeks_horizon(52), 52);
        assert_eq!(clamp_weeks_horizon(60), 52);
    }

    /// Round-trip: from_response ∘ to_put_body ist (mit leerem token_input) die
    /// Identität auf allen persistierbaren Feldern — d. h. eine Save-Then-Reload
    /// Runde ändert die Form nicht (außer dass token_input leer bleibt, was
    /// die einzige nicht-persistierte Änderung ist).
    #[test]
    fn pdf_export_form_save_reload_roundtrip_preserves_fields() {
        let mut form = PdfExportForm::default();
        form.enabled = true;
        form.nextcloud_url = "https://x/dav".to_string();
        form.webdav_user = "u".to_string();
        form.target_folder = "folder".to_string();
        form.weeks_horizon = 12;
        form.cron_schedule = "0 5 * * 1".to_string();
        // token_input leer → Backend würde bestehenden Token behalten;
        // in der From<&PdfExportConfig> maskiert wird der Token wieder auf
        // None gesetzt → Response-Body hat webdav_app_token = None.
        let put_body = pdf_export_form_to_put_body(&form);
        // Simuliert die Server-Round-trip: die Response ist der PUT-Body
        // MIT webdav_app_token = None (Server maskiert immer).
        let mut response = put_body.clone();
        response.webdav_app_token = None;
        let reloaded = pdf_export_form_from_response(&response);
        assert_eq!(reloaded.enabled, form.enabled);
        assert_eq!(reloaded.nextcloud_url, form.nextcloud_url);
        assert_eq!(reloaded.webdav_user, form.webdav_user);
        assert_eq!(reloaded.target_folder, form.target_folder);
        assert_eq!(reloaded.weeks_horizon, form.weeks_horizon);
        assert_eq!(reloaded.cron_schedule, form.cron_schedule);
        assert_eq!(reloaded.token_input, ""); // token_input bleibt leer
    }
}
