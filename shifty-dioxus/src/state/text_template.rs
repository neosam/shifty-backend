use std::rc::Rc;
use uuid::Uuid;
use rest_types::{TextTemplateTO, CreateTextTemplateRequestTO, UpdateTextTemplateRequestTO, TemplateEngineTO};

#[derive(Clone, Debug, PartialEq)]
pub enum TemplateEngine {
    Tera,
    MiniJinja,
}

impl From<&TemplateEngineTO> for TemplateEngine {
    fn from(engine: &TemplateEngineTO) -> Self {
        match engine {
            TemplateEngineTO::Tera => TemplateEngine::Tera,
            TemplateEngineTO::MiniJinja => TemplateEngine::MiniJinja,
        }
    }
}

impl From<&TemplateEngine> for TemplateEngineTO {
    fn from(engine: &TemplateEngine) -> Self {
        match engine {
            TemplateEngine::Tera => TemplateEngineTO::Tera,
            TemplateEngine::MiniJinja => TemplateEngineTO::MiniJinja,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextTemplate {
    pub id: Uuid,
    pub name: Option<Rc<str>>,
    pub template_type: Rc<str>,
    pub template_text: Rc<str>,
    pub template_engine: TemplateEngine,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub created_by: Option<Rc<str>>,
}

impl From<&TextTemplateTO> for TextTemplate {
    fn from(template: &TextTemplateTO) -> Self {
        Self {
            id: template.id,
            name: template.name.as_ref().map(|s| s.to_string().into()),
            template_type: template.template_type.to_string().into(),
            template_text: template.template_text.to_string().into(),
            template_engine: TemplateEngine::from(&template.template_engine),
            created_at: template.created_at,
            created_by: template.created_by.as_ref().map(|s| s.to_string().into()),
        }
    }
}

impl TextTemplate {
    pub fn to_create_request(&self) -> CreateTextTemplateRequestTO {
        CreateTextTemplateRequestTO {
            name: self.name.as_ref().map(|s| s.to_string().into()),
            template_type: self.template_type.to_string().into(),
            template_text: self.template_text.to_string().into(),
            template_engine: TemplateEngineTO::from(&self.template_engine),
        }
    }

    pub fn to_update_request(&self) -> UpdateTextTemplateRequestTO {
        UpdateTextTemplateRequestTO {
            name: self.name.as_ref().map(|s| s.to_string().into()),
            template_type: self.template_type.to_string().into(),
            template_text: self.template_text.to_string().into(),
            template_engine: TemplateEngineTO::from(&self.template_engine),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_from_to_tera() {
        let to = TemplateEngineTO::Tera;
        let engine = TemplateEngine::from(&to);
        assert_eq!(engine, TemplateEngine::Tera);
        let back = TemplateEngineTO::from(&engine);
        assert_eq!(back, TemplateEngineTO::Tera);
    }

    #[test]
    fn test_template_engine_from_to_minijinja() {
        let to = TemplateEngineTO::MiniJinja;
        let engine = TemplateEngine::from(&to);
        assert_eq!(engine, TemplateEngine::MiniJinja);
        let back = TemplateEngineTO::from(&engine);
        assert_eq!(back, TemplateEngineTO::MiniJinja);
    }

    #[test]
    fn test_text_template_from_to() {
        let to = TextTemplateTO {
            id: Uuid::nil(),
            name: Some("test".into()),
            template_type: "billing-period".into(),
            template_text: "hello".into(),
            template_engine: TemplateEngineTO::MiniJinja,
            created_at: None,
            created_by: None,
            deleted: None,
            deleted_by: None,
            version: Uuid::nil(),
        };
        let template = TextTemplate::from(&to);
        assert_eq!(template.template_engine, TemplateEngine::MiniJinja);
        assert_eq!(template.name, Some(Rc::from("test")));
    }

    #[test]
    fn test_to_create_request_includes_engine() {
        let template = TextTemplate {
            id: Uuid::nil(),
            name: None,
            template_type: "billing-period".into(),
            template_text: "hello".into(),
            template_engine: TemplateEngine::MiniJinja,
            created_at: None,
            created_by: None,
        };
        let req = template.to_create_request();
        assert_eq!(req.template_engine, TemplateEngineTO::MiniJinja);
    }

    #[test]
    fn test_to_update_request_includes_engine() {
        let template = TextTemplate {
            id: Uuid::nil(),
            name: Some("test".into()),
            template_type: "billing-period".into(),
            template_text: "hello".into(),
            template_engine: TemplateEngine::Tera,
            created_at: None,
            created_by: None,
        };
        let req = template.to_update_request();
        assert_eq!(req.template_engine, TemplateEngineTO::Tera);
    }

    #[test]
    fn test_template_engine_to_serde_tera() {
        let json = serde_json::to_string(&TemplateEngineTO::Tera).unwrap();
        assert_eq!(json, "\"tera\"");
        let engine: TemplateEngineTO = serde_json::from_str("\"tera\"").unwrap();
        assert_eq!(engine, TemplateEngineTO::Tera);
    }

    #[test]
    fn test_template_engine_to_serde_minijinja() {
        let json = serde_json::to_string(&TemplateEngineTO::MiniJinja).unwrap();
        assert_eq!(json, "\"minijinja\"");
        let engine: TemplateEngineTO = serde_json::from_str("\"minijinja\"").unwrap();
        assert_eq!(engine, TemplateEngineTO::MiniJinja);
    }

    #[test]
    fn test_template_engine_to_serde_roundtrip() {
        for engine in [TemplateEngineTO::Tera, TemplateEngineTO::MiniJinja] {
            let json = serde_json::to_string(&engine).unwrap();
            let deserialized: TemplateEngineTO = serde_json::from_str(&json).unwrap();
            assert_eq!(engine, deserialized);
        }
    }

    #[test]
    fn test_text_template_to_with_engine_json_roundtrip() {
        let json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "template_type": "billing-period",
            "template_text": "hello",
            "template_engine": "minijinja"
        });
        let template: TextTemplateTO = serde_json::from_value(json).unwrap();
        assert_eq!(template.template_engine, TemplateEngineTO::MiniJinja);

        let serialized = serde_json::to_value(&template).unwrap();
        assert_eq!(serialized["template_engine"], "minijinja");
    }

    #[test]
    fn test_create_request_to_engine_serialization() {
        use rest_types::CreateTextTemplateRequestTO;
        let req = CreateTextTemplateRequestTO {
            name: None,
            template_type: "billing-period".into(),
            template_text: "test".into(),
            template_engine: TemplateEngineTO::Tera,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["template_engine"], "tera");
    }

    #[test]
    fn test_update_request_to_engine_serialization() {
        use rest_types::UpdateTextTemplateRequestTO;
        let req = UpdateTextTemplateRequestTO {
            name: Some("name".into()),
            template_type: "billing-period".into(),
            template_text: "test".into(),
            template_engine: TemplateEngineTO::MiniJinja,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["template_engine"], "minijinja");
    }

    #[test]
    fn test_text_template_to_deserialize_without_engine_defaults_to_tera() {
        let json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "template_type": "billing-period",
            "template_text": "hello"
        });
        let template: TextTemplateTO = serde_json::from_value(json).unwrap();
        assert_eq!(template.template_engine, TemplateEngineTO::Tera);
    }
}